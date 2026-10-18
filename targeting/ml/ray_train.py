"""Ray distributed training orchestration for 6Hit DQN — Multi-Trainer + Parameter Server.

§3.6.1: "Training data is collected from Flink execution feedback,
with the model retrained on a continuous schedule using Ray for
distributed training orchestration."

Architecture:
- 2+ DQNTrainer actors (data parallel workers)
- 1 ParameterServer actor (weight aggregation + synchronization)
- Periodic sync: trainers pull global weights, train locally, push gradients
"""

from __future__ import annotations

import os
import time
import threading
from dataclasses import dataclass, field
from typing import Optional

import torch
import ray

try:
    from .dqn_model import DQNAgent, DuelingDQN, STATE_DIM, ACTION_DIM
except ImportError:
    from dqn_model import DQNAgent, DuelingDQN, STATE_DIM, ACTION_DIM


# ─────────────────────────────────────────────────────────
# Parameter Server Actor
# ─────────────────────────────────────────────────────────

@ray.remote
class ParameterServer:
    """Centralized parameter server for weight aggregation.

    Implements FedAvg-style aggregation: collects state_dicts from trainers,
    averages them, and broadcasts back.
    """

    def __init__(
        self,
        state_dim: int = STATE_DIM,
        action_dim: int = ACTION_DIM,
        checkpoint_dir: str = "/tmp/aegis/checkpoints",
    ):
        self.global_model = DuelingDQN(state_dim=state_dim, action_dim=action_dim)
        self.checkpoint_dir = checkpoint_dir
        os.makedirs(checkpoint_dir, exist_ok=True)
        self.trainer_states: dict[int, dict] = {}
        self.sync_count = 0
        self._lock = threading.Lock()

    def register_trainer(self, trainer_id: int, state_dict: dict):
        """Register initial trainer weights."""
        with self._lock:
            self.trainer_states[trainer_id] = state_dict
            if self.sync_count == 0:
                # Initialize global model with first trainer's weights
                self.global_model.load_state_dict(state_dict)

    def push_weights(self, trainer_id: int, state_dict: dict) -> dict:
        """Receive weights from trainer, aggregate, return global weights."""
        with self._lock:
            self.trainer_states[trainer_id] = state_dict
            # FedAvg: simple average of all registered trainers
            keys = list(state_dict.keys())
            avg_state = {}
            for k in keys:
                avg_state[k] = sum(s[k] for s in self.trainer_states.values()) / len(self.trainer_states)
            self.global_model.load_state_dict(avg_state)
            self.sync_count += 1
            return avg_state

    def pull_weights(self) -> dict:
        """Get current global weights."""
        with self._lock:
            return self.global_model.state_dict()

    def checkpoint(self, tag: str = "global") -> str:
        """Save global model checkpoint."""
        path = os.path.join(self.checkpoint_dir, f"dqn_global_{tag}.pt")
        torch.save({"model": self.global_model.state_dict(), "sync_count": self.sync_count}, path)
        return path

    def get_metrics(self) -> dict:
        return {
            "num_trainers": len(self.trainer_states),
            "sync_count": self.sync_count,
            "checkpoint_dir": self.checkpoint_dir,
        }


# ─────────────────────────────────────────────────────────
# DQN Trainer Actor (Data Parallel Worker)
# ─────────────────────────────────────────────────────────

@ray.remote
class DQNTrainer:
    """Ray actor for distributed DQN training worker.

    Each trainer:
    - Maintains local replay buffer
    - Trains on local data
    - Periodically syncs with ParameterServer
    """

    def __init__(
        self,
        trainer_id: int,
        parameter_server: ray.ObjectRef,
        state_dim: int = STATE_DIM,
        action_dim: int = ACTION_DIM,
        checkpoint_dir: str = "/tmp/aegis/checkpoints",
        sync_interval: int = 50,
    ):
        self.trainer_id = trainer_id
        self.parameter_server = parameter_server
        self.sync_interval = sync_interval
        self.checkpoint_dir = checkpoint_dir
        os.makedirs(checkpoint_dir, exist_ok=True)

        self.agent = DQNAgent(state_dim=state_dim, action_dim=action_dim)
        self.steps_since_sync = 0
        self.local_steps = 0
        self.episode_rewards: list[float] = []
        self.loss_history: list[float] = []

        # Register with parameter server
        ray.get(self.parameter_server.register_trainer.remote(
            trainer_id, self.agent.policy_net.state_dict()
        ))

    def store_transition(
        self,
        state: list[float],
        action: int,
        reward: float,
        next_state: list[float],
        done: bool,
    ):
        """Store a transition in local replay buffer."""
        import torch
        s = torch.tensor(state, dtype=torch.float32)
        ns = torch.tensor(next_state, dtype=torch.float32)
        self.agent.store_transition(s, action, reward, ns, done)

    def train(self, num_steps: int = 1) -> list[float]:
        """Perform local training steps."""
        losses = []
        for _ in range(num_steps):
            loss = self.agent.train_step()
            if loss is not None:
                losses.append(loss)
                self.loss_history.append(loss)
                self.local_steps += 1
                self.steps_since_sync += 1

                # Sync with parameter server
                if self.steps_since_sync >= self.sync_interval:
                    self._sync_with_ps()
                    self.steps_since_sync = 0
        return losses

    def _sync_with_ps(self):
        """Synchronize weights with parameter server."""
        # Push local weights, pull global weights
        local_state = self.agent.policy_net.state_dict()
        global_state = ray.get(self.parameter_server.push_weights.remote(self.trainer_id, local_state))
        self.agent.policy_net.load_state_dict(global_state)
        self.agent.target_net.load_state_dict(global_state)  # Also sync target

    def force_sync(self):
        """Force synchronization with parameter server."""
        self._sync_with_ps()
        self.steps_since_sync = 0

    def checkpoint(self, tag: str = "trainer") -> str:
        """Save local model checkpoint."""
        path = os.path.join(self.checkpoint_dir, f"dqn_trainer{self.trainer_id}_{tag}.pt")
        self.agent.save(path)
        return path

    def get_metrics(self) -> dict:
        return {
            "trainer_id": self.trainer_id,
            "local_steps": self.local_steps,
            "buffer_size": len(self.agent.buffer),
            "avg_loss_100": (
                sum(self.loss_history[-100:]) / len(self.loss_history[-100:])
                if self.loss_history else 0.0
            ),
            "avg_reward_100": (
                sum(self.episode_rewards[-100:]) / len(self.episode_rewards[-100:])
                if self.episode_rewards else 0.0
            ),
        }


# ─────────────────────────────────────────────────────────
# Distributed Training Loop
# ─────────────────────────────────────────────────────────

@dataclass
class DistributedTrainingConfig:
    num_trainers: int = 3
    num_parameter_servers: int = 1
    transitions_per_step_per_trainer: int = 256
    train_batch_size: int = 64
    sync_interval: int = 50
    checkpoint_interval: int = 100
    max_steps: int = 10000
    run_duration_sec: int = 1800  # 30 minutes default


def run_distributed_training(config: DistributedTrainingConfig | None = None):
    """Run distributed training with multiple trainers + parameter server."""
    if config is None:
        config = DistributedTrainingConfig()

    ray.init(ignore_reinit_error=True, include_dashboard=False)

    print(f"[Ray] Starting distributed training:")
    print(f"  Trainers: {config.num_trainers}")
    print(f"  Parameter Servers: {config.num_parameter_servers}")
    print(f"  Dashboard: http://127.0.0.1:8265")
    print(f"  Target duration: {config.run_duration_sec}s ({config.run_duration_sec/60:.0f} min)")

    # Create parameter server
    ps = ParameterServer.remote()
    print("[Ray] ParameterServer created")

    # Create trainers
    trainers = [
        DQNTrainer.remote(
            trainer_id=i,
            parameter_server=ps,
            sync_interval=config.sync_interval,
        )
        for i in range(config.num_trainers)
    ]
    print(f"[Ray] Created {len(trainers)} trainers")

    # Wait for registration
    time.sleep(2)

    start_time = time.time()
    step = 0

    try:
        while True:
            # Check duration limit
            elapsed = time.time() - start_time
            if elapsed > config.run_duration_sec:
                print(f"[Ray] Duration limit reached ({elapsed:.0f}s)")
                break

            if step >= config.max_steps:
                print(f"[Ray] Max steps reached ({step})")
                break

            # Generate transitions for each trainer (simulated PoP data)
            futures = []
            for trainer in trainers:
                for _ in range(config.transitions_per_step_per_trainer):
                    # Simulate diverse state observations
                    import random
                    state = [random.random() for _ in range(STATE_DIM)]
                    action = random.randint(0, ACTION_DIM - 1)
                    reward = random.uniform(-1.0, 2.0)  # -1 to +2
                    next_state = [random.random() for _ in range(STATE_DIM)]
                    done = random.random() < 0.01

                    futures.append(trainer.store_transition.remote(
                        state, action, reward, next_state, done
                    ))

            # Wait for all transitions to be stored
            ray.get(futures)

            # Train each trainer
            train_futures = [t.train.remote(config.train_batch_size) for t in trainers]
            all_losses = ray.get(train_futures)

            step += 1

            # Logging
            if step % 10 == 0:
                avg_losses = [sum(l)/len(l) if l else 0 for l in all_losses]
                print(f"[Step {step} | {elapsed:.0f}s] losses={['%.4f'%x for x in avg_losses]}")

                # Get trainer metrics
                metrics_futures = [t.get_metrics.remote() for t in trainers]
                all_metrics = ray.get(metrics_futures)
                for m in all_metrics:
                    print(f"  Trainer {m['trainer_id']}: steps={m['local_steps']} "
                          f"buffer={m['buffer_size']} avg_loss={m['avg_loss_100']:.4f}")

                # PS metrics
                ps_metrics = ray.get(ps.get_metrics.remote())
                print(f"  PS: syncs={ps_metrics['sync_count']} trainers={ps_metrics['num_trainers']}")

            # Checkpointing
            if step % config.checkpoint_interval == 0:
                ps_path = ray.get(ps.checkpoint.remote(f"step_{step}"))
                print(f"[Checkpoint] Global: {ps_path}")

                for trainer in trainers:
                    t_path = ray.get(trainer.checkpoint.remote(f"step_{step}"))
                    print(f"[Checkpoint] {t_path}")

    except KeyboardInterrupt:
        print("\n[Ray] Interrupted, saving final checkpoints...")

    finally:
        # Final checkpoints
        ps_path = ray.get(ps.checkpoint.remote("final"))
        print(f"[Final] Global checkpoint: {ps_path}")

        for trainer in trainers:
            t_path = ray.get(trainer.checkpoint.remote("final"))
            print(f"[Final] Trainer {trainer.trainer_id} checkpoint: {t_path}")

# Print final metrics
        print("\n=== FINAL METRICS ===")
        ps_metrics = ray.get(ps.get_metrics.remote())
        print(f"ParameterServer: {ps_metrics}")

        for i, trainer in enumerate(trainers):
            m = ray.get(trainer.get_metrics.remote())
            t_path = ray.get(trainer.checkpoint.remote("final"))
            print(f"Trainer {i}: {m}")
            print(f"[Final] Trainer {i} checkpoint: {t_path}")

        ray.shutdown()
        print("[Ray] Shutdown complete")


# ─────────────────────────────────────────────────────────
# CLI
# ─────────────────────────────────────────────────────────

def main():
    import argparse

    parser = argparse.ArgumentParser(description="6Hit DQN Distributed Training")
    parser.add_argument("--trainers", type=int, default=3, help="Number of trainer actors")
    parser.add_argument("--duration", type=int, default=1800, help="Run duration in seconds")
    parser.add_argument("--sync-interval", type=int, default=50, help="Sync interval (steps)")
    parser.add_argument("--checkpoint-interval", type=int, default=100, help="Checkpoint interval")
    args = parser.parse_args()

    config = DistributedTrainingConfig(
        num_trainers=args.trainers,
        run_duration_sec=args.duration,
        sync_interval=args.sync_interval,
        checkpoint_interval=args.checkpoint_interval,
    )

    print("[AEGIS] Starting Ray distributed training for 6Hit DQN")
    run_distributed_training(config)


if __name__ == "__main__":
    main()