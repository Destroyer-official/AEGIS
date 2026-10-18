"""6Hit DQN Model Definition — IPv6 Target Traversal

§3.6.1: "The model uses a Deep Q-Network (DQN) architecture where:
  - State: Encoded subnet prefix features, historical hit rates, BGP path properties
  - Action: Selection of specific /64 or /120 address subranges
  - Reward: Discovery of live, non-alias hosts (+1) vs. barren subnets or alias patterns (-1)"

Reference: 6Hit (Hou et al., INFOCOM 2021)
"""

from __future__ import annotations

import torch
import torch.nn as nn
import torch.nn.functional as F


# ─────────────────────────────────────────────────────────
# State Encoding
# ─────────────────────────────────────────────────────────

# State feature dimensions (mirrors Rust State struct)
STATE_DIM = 12
# [prefix_len, hit_rate, as_path_len, origin_asn_norm,
#  address_density, alloc_granularity, probe_budget_remaining_norm,
#  seed_count_norm, expected_reward, region_x, region_y, region_z]

ACTION_DIM = 4
# [ProbeSlash64, ProbeSlash120, Skip, Repartition]


# ─────────────────────────────────────────────────────────
# Dueling DQN Architecture
# ─────────────────────────────────────────────────────────

class DuelingDQN(nn.Module):
    """Dueling DQN for IPv6 target traversal.

    Separates state-value and advantage streams to improve learning
    stability in regions with uniform action values.

    Architecture:
        Input (STATE_DIM) -> FC(128) -> ReLU -> FC(128) -> ReLU
            -> Value stream:     FC(64) -> ReLU -> FC(1)
            -> Advantage stream: FC(64) -> ReLU -> FC(ACTION_DIM)
            -> Q(s,a) = V(s) + A(s,a) - mean(A(s,:))
    """

    def __init__(self, state_dim: int = STATE_DIM, action_dim: int = ACTION_DIM):
        super().__init__()
        self.state_dim = state_dim
        self.action_dim = action_dim

        # Shared feature extractor
        self.feature = nn.Sequential(
            nn.Linear(state_dim, 128),
            nn.ReLU(),
            nn.Linear(128, 128),
            nn.ReLU(),
        )

        # Value stream
        self.value_stream = nn.Sequential(
            nn.Linear(128, 64),
            nn.ReLU(),
            nn.Linear(64, 1),
        )

        # Advantage stream
        self.advantage_stream = nn.Sequential(
            nn.Linear(128, 64),
            nn.ReLU(),
            nn.Linear(64, action_dim),
        )

    def forward(self, state: torch.Tensor) -> torch.Tensor:
        """Compute Q-values for all actions given a batch of states.

        Args:
            state: (batch, state_dim) tensor

        Returns:
            Q-values: (batch, action_dim) tensor
        """
        features = self.feature(state)
        value = self.value_stream(features)
        advantage = self.advantage_stream(features)
        # Dueling aggregation
        q = value + advantage - advantage.mean(dim=-1, keepdim=True)
        return q

    def act(self, state: torch.Tensor, epsilon: float = 0.0) -> int:
        """Select action using epsilon-greedy policy.

        Args:
            state: (state_dim,) tensor — single state observation
            epsilon: exploration rate

        Returns:
            Action index (0..action_dim-1)
        """
        if torch.rand(1).item() < epsilon:
            return torch.randint(0, self.action_dim, (1,)).item()

        with torch.no_grad():
            state = state.unsqueeze(0)
            q_values = self.forward(state)
            return q_values.argmax(dim=-1).item()


# ─────────────────────────────────────────────────────────
# Double DQN Target Network
# ─────────────────────────────────────────────────────────

class DQNAgent:
    """Double DQN agent with target network and experience replay.

    §3.6.1: Training data collected from Flink execution feedback,
    model retrained on continuous schedule using Ray.
    """

    def __init__(
        self,
        state_dim: int = STATE_DIM,
        action_dim: int = ACTION_DIM,
        lr: float = 1e-4,
        gamma: float = 0.99,
        tau: float = 0.005,
        buffer_size: int = 100_000,
        batch_size: int = 64,
    ):
        self.gamma = gamma
        self.tau = tau
        self.batch_size = batch_size

        self.policy_net = DuelingDQN(state_dim, action_dim)
        self.target_net = DuelingDQN(state_dim, action_dim)
        self.target_net.load_state_dict(self.policy_net.state_dict())
        self.target_net.eval()

        self.optimizer = torch.optim.Adam(self.policy_net.parameters(), lr=lr)
        self.buffer: list[tuple[torch.Tensor, int, float, torch.Tensor, bool]] = []
        self.buffer_size = buffer_size
        self.steps = 0

    def act(self, state: torch.Tensor, epsilon: float = 0.0) -> int:
        """Select action using epsilon-greedy policy.

        Args:
            state: (state_dim,) tensor — single state observation
            epsilon: exploration rate

        Returns:
            Action index (0..action_dim-1)
        """
        return self.policy_net.act(state, epsilon)

    def store_transition(
        self,
        state: torch.Tensor,
        action: int,
        reward: float,
        next_state: torch.Tensor,
        done: bool,
    ):
        """Store a transition in the replay buffer."""
        if len(self.buffer) >= self.buffer_size:
            self.buffer.pop(0)
        self.buffer.append((state, action, reward, next_state, done))

    def train_step(self) -> float | None:
        """Sample a batch and perform one gradient step.

        Returns loss value or None if buffer too small.
        """
        if len(self.buffer) < self.batch_size:
            return None

        import random
        batch = random.sample(self.buffer, self.batch_size)

        states = torch.stack([t[0] for t in batch])
        actions = torch.tensor([t[1] for t in batch], dtype=torch.long)
        rewards = torch.tensor([t[2] for t in batch], dtype=torch.float32)
        next_states = torch.stack([t[3] for t in batch])
        dones = torch.tensor([t[4] for t in batch], dtype=torch.float32)

        # Current Q-values
        q_values = self.policy_net(states)
        q_value = q_values.gather(1, actions.unsqueeze(1)).squeeze(1)

        # Double DQN: use policy net to select action, target net to evaluate
        with torch.no_grad():
            next_actions = self.policy_net(next_states).argmax(dim=1)
            next_q_values = self.target_net(next_states)
            next_q_value = next_q_values.gather(1, next_actions.unsqueeze(1)).squeeze(1)
            target = rewards + self.gamma * next_q_value * (1 - dones)

        loss = F.smooth_l1_loss(q_value, target)

        self.optimizer.zero_grad()
        loss.backward()
        torch.nn.utils.clip_grad_norm_(self.policy_net.parameters(), 1.0)
        self.optimizer.step()

        # Soft update target network
        self.steps += 1
        if self.steps % 10 == 0:
            self._soft_update()

        return loss.item()

    def _soft_update(self):
        """Polyak averaging for target network."""
        for tp, pp in zip(self.target_net.parameters(), self.policy_net.parameters()):
            tp.data.copy_(self.tau * pp.data + (1.0 - self.tau) * tp.data)

    def save(self, path: str):
        torch.save({
            "policy": self.policy_net.state_dict(),
            "target": self.target_net.state_dict(),
            "optimizer": self.optimizer.state_dict(),
            "steps": self.steps,
        }, path)

    def load(self, path: str):
        checkpoint = torch.load(path, map_location="cpu", weights_only=True)
        self.policy_net.load_state_dict(checkpoint["policy"])
        self.target_net.load_state_dict(checkpoint["target"])
        self.optimizer.load_state_dict(checkpoint["optimizer"])
        self.steps = checkpoint["steps"]


# ─────────────────────────────────────────────────────────
# State Encoding Helpers
# ─────────────────────────────────────────────────────────

def encode_state(
    prefix_len: int,
    hit_rate: float,
    as_path_len: int,
    origin_asn: int,
    address_density: float,
    alloc_granularity: str,
    probe_budget_remaining: int,
    seed_count: int,
    expected_reward: float,
    region_coords: tuple[float, float, float] = (0.0, 0.0, 0.0),
) -> torch.Tensor:
    """Encode a raw state observation into the fixed-size state tensor.

    This bridges the Rust State struct fields into the PyTorch model input.
    """
    alloc_map = {
        "Host": 0.0,
        "Slash120": 0.25,
        "Slash64": 0.5,
        "Slash48": 0.75,
        "Slash32": 1.0,
        "Unknown": 0.5,
    }

    return torch.tensor([
        prefix_len / 128.0,
        hit_rate,
        as_path_len / 32.0,
        min(origin_asn / 65535.0, 1.0),
        address_density,
        alloc_map.get(alloc_granularity, 0.5),
        min(probe_budget_remaining / 10000.0, 1.0),
        min(seed_count / 1000.0, 1.0),
        min(expected_reward / 100.0, 1.0),
        region_coords[0],
        region_coords[1],
        region_coords[2],
    ], dtype=torch.float32)
