"""Auto-Retraining Pipeline — Nightly DQN Model Retraining

§3.6.1: "Training data is collected from Flink execution feedback,
with the model retrained on a continuous schedule."

Pipeline:
1. Sample new hits from recent data
2. Retrain DQN with new data + replay buffer
3. Validate performance on held-out set
4. Promote if reward improves by >= 5%
5. Export new ONNX model for Triton

Zero external dependencies: local training only, uses dqn_model.py.
"""

from __future__ import annotations

import json
import math
import os
import random
import time
from dataclasses import dataclass, field
from datetime import datetime
from typing import List, Optional, Tuple

import torch
import torch.nn as nn
import torch.optim as optim

try:
    from .dqn_model import (
        ACTION_DIM,
        STATE_DIM,
        DQNAgent,
        DuelingDQN,
        encode_state,
    )
except ImportError:
    from dqn_model import (
        ACTION_DIM,
        STATE_DIM,
        DQNAgent,
        DuelingDQN,
        encode_state,
    )


# ─────────────────────────────────────────────────────────
# Training Data Structures
# ─────────────────────────────────────────────────────────

@dataclass
class TrainingSample:
    """A single (state, action, reward, next_state, done) transition."""
    prefix_len: int
    hit_rate: float
    as_path_len: int
    origin_asn: int
    address_density: float
    alloc_granularity: str
    probe_budget_remaining: int
    seed_count: int
    expected_reward: float
    action: int
    reward: float
    next_prefix_len: int
    next_hit_rate: float
    next_as_path_len: int
    next_origin_asn: int
    next_address_density: float
    next_alloc_granularity: str
    next_probe_budget_remaining: int
    next_seed_count: int
    next_expected_reward: float
    done: bool


@dataclass
class RetrainResult:
    """Result of an auto-retrain run."""
    status: str  # "promoted", "rejected", "error"
    old_reward: float
    new_reward: float
    improvement_pct: float
    epochs_trained: int
    samples_used: int
    checkpoint_path: str
    onnx_path: Optional[str]
    timestamp: str
    details: str


# ─────────────────────────────────────────────────────────
# Data Sampler
# ─────────────────────────────────────────────────────────

def load_training_data(
    data_dir: str = "/tmp/aegis/training_data",
    max_samples: int = 10000,
) -> List[TrainingSample]:
    """Load recent training samples from data directory."""
    samples = []
    if not os.path.exists(data_dir):
        return samples

    for fname in sorted(os.listdir(data_dir), reverse=True)[:50]:
        if not fname.endswith(".jsonl"):
            continue
        fpath = os.path.join(data_dir, fname)
        with open(fpath, "r") as f:
            for line in f:
                if len(samples) >= max_samples:
                    break
                line = line.strip()
                if not line:
                    continue
                try:
                    entry = json.loads(line)
                    sample = TrainingSample(**entry)
                    samples.append(sample)
                except (json.JSONDecodeError, TypeError):
                    continue
    return samples


def generate_synthetic_data(
    num_samples: int = 5000,
    seed: int = 42,
) -> List[TrainingSample]:
    """Generate synthetic training data for bootstrap / testing."""
    rng = random.Random(seed)
    samples = []

    for _ in range(num_samples):
        prefix_len = rng.randint(32, 128)
        hit_rate = rng.random()
        as_path_len = rng.randint(1, 10)
        origin_asn = rng.randint(1000, 65000)
        address_density = rng.random()
        alloc_gran = rng.choice(["Slash64", "Slash120", "Unknown"])
        budget = rng.randint(0, 10000)
        seed_cnt = rng.randint(0, 200)
        exp_rew = rng.uniform(-5.0, 20.0)

        action = rng.randint(0, ACTION_DIM - 1)
        reward = rng.uniform(-1.0, 1.0)

        samples.append(TrainingSample(
            prefix_len=prefix_len,
            hit_rate=hit_rate,
            as_path_len=as_path_len,
            origin_asn=origin_asn,
            address_density=address_density,
            alloc_granularity=alloc_gran,
            probe_budget_remaining=budget,
            seed_count=seed_cnt,
            expected_reward=exp_rew,
            action=action,
            reward=reward,
            next_prefix_len=min(128, prefix_len + rng.choice([-4, 0, 4])),
            next_hit_rate=min(1.0, max(0.0, hit_rate + rng.uniform(-0.1, 0.1))),
            next_as_path_len=max(1, as_path_len + rng.choice([-1, 0, 1])),
            next_origin_asn=origin_asn + rng.choice([0, 0, rng.randint(-100, 100)]),
            next_address_density=min(1.0, max(0.0, address_density + rng.uniform(-0.05, 0.05))),
            next_alloc_granularity=alloc_gran,
            next_probe_budget_remaining=max(0, budget - rng.randint(0, 500)),
            next_seed_count=max(0, seed_cnt + rng.randint(-5, 5)),
            next_expected_reward=exp_rew + reward,
            done=rng.random() < 0.05,
        ))
    return samples


# ─────────────────────────────────────────────────────────
# Auto-Retrainer
# ─────────────────────────────────────────────────────────

class AutoRetrainer:
    """Nightly auto-retraining pipeline for 6Hit DQN.

    Samples new data, retrains, validates, and promotes if reward improves.
    """

    def __init__(
        self,
        checkpoint_dir: str = "/tmp/aegis/checkpoints",
        training_data_dir: str = "/tmp/aegis/training_data",
        promote_threshold: float = 0.05,
        max_epochs: int = 20,
        batch_size: int = 64,
        lr: float = 1e-4,
        gamma: float = 0.99,
        tau: float = 0.005,
    ):
        self.checkpoint_dir = checkpoint_dir
        self.training_data_dir = training_data_dir
        self.promote_threshold = promote_threshold
        self.max_epochs = max_epochs
        self.batch_size = batch_size
        self.lr = lr
        self.gamma = gamma
        self.tau = tau
        os.makedirs(checkpoint_dir, exist_ok=True)

    def load_current_model(self) -> Tuple[DuelingDQN, float]:
        """Load current production model and its reward."""
        model = DuelingDQN(state_dim=STATE_DIM, action_dim=ACTION_DIM)
        ckpt_path = os.path.join(self.checkpoint_dir, "dqn_global_final.pt")
        reward = 0.0

        if os.path.exists(ckpt_path):
            ckpt = torch.load(ckpt_path, map_location="cpu", weights_only=True)
            state_dict = ckpt.get("policy", ckpt.get("model", ckpt))
            model.load_state_dict(state_dict)
            reward = ckpt.get("avg_reward", ckpt.get("reward", 0.0))
        return model, reward

    def train_model(
        self,
        model: DuelingDQN,
        samples: List[TrainingSample],
    ) -> Tuple[DuelingDQN, float, int]:
        """Train DQN on samples, return trained model, avg reward, epochs."""
        agent = DQNAgent(state_dim=STATE_DIM, action_dim=ACTION_DIM)
        agent.policy_net.load_state_dict(model.state_dict())
        agent.target_net.load_state_dict(model.state_dict())

        for epoch in range(self.max_epochs):
            random.shuffle(samples)
            epoch_rewards = []

            for i in range(0, len(samples), self.batch_size):
                batch = samples[i:i + self.batch_size]
                for s in batch:
                    state = encode_state(
                        s.prefix_len, s.hit_rate, s.as_path_len,
                        s.origin_asn, s.address_density, s.alloc_granularity,
                        s.probe_budget_remaining, s.seed_count, s.expected_reward,
                    )
                    next_state = encode_state(
                        s.next_prefix_len, s.next_hit_rate, s.next_as_path_len,
                        s.next_origin_asn, s.next_address_density,
                        s.next_alloc_granularity, s.next_probe_budget_remaining,
                        s.next_seed_count, s.next_expected_reward,
                    )
                    agent.store_transition(state, s.action, s.reward, next_state, s.done)
                    epoch_rewards.append(s.reward)

                agent.train_step()
                agent._soft_update()

        avg_reward = sum(epoch_rewards) / len(epoch_rewards) if epoch_rewards else 0.0
        trained_model = DuelingDQN(state_dim=STATE_DIM, action_dim=ACTION_DIM)
        trained_model.load_state_dict(agent.policy_net.state_dict())
        return trained_model, avg_reward, self.max_epochs

    def evaluate_model(self, model: DuelingDQN, samples: List[TrainingSample]) -> float:
        """Evaluate model on held-out samples, return avg Q-value."""
        model.eval()
        total_q = 0.0
        count = 0

        with torch.no_grad():
            for s in samples[:500]:
                state = encode_state(
                    s.prefix_len, s.hit_rate, s.as_path_len,
                    s.origin_asn, s.address_density, s.alloc_granularity,
                    s.probe_budget_remaining, s.seed_count, s.expected_reward,
                )
                q_vals = model(state.unsqueeze(0))
                total_q += q_vals.max().item()
                count += 1

        return total_q / count if count > 0 else 0.0

    def save_checkpoint(self, model: DuelingDQN, reward: float, tag: str) -> str:
        """Save model checkpoint."""
        path = os.path.join(self.checkpoint_dir, f"dqn_{tag}.pt")
        torch.save({
            "policy": model.state_dict(),
            "avg_reward": reward,
            "timestamp": datetime.now().isoformat(),
            "state_dim": STATE_DIM,
            "action_dim": ACTION_DIM,
        }, path)
        return path

    def export_onnx(self, model: DuelingDQN, output_path: str) -> str:
        """Export trained model to ONNX."""
        model.eval()
        dummy = torch.randn(1, STATE_DIM, dtype=torch.float32)
        output_path = os.path.normpath(os.path.abspath(output_path))
        os.makedirs(os.path.dirname(output_path) or ".", exist_ok=True)

        torch.onnx.export(
            model,
            dummy,
            output_path,
            export_params=True,
            opset_version=17,
            do_constant_folding=True,
            input_names=["input"],
            output_names=["output"],
            dynamic_axes={
                "input": {0: "batch_size"},
                "output": {0: "batch_size"},
            },
            verbose=False,
        )
        return output_path

    def run(self) -> RetrainResult:
        """Execute the full retrain pipeline."""
        timestamp = datetime.now().isoformat()

        # Load current model
        current_model, old_reward = self.load_current_model()
        current_model.eval()

        # Load training data (fall back to synthetic)
        samples = load_training_data(self.training_data_dir)
        if len(samples) < 100:
            samples = generate_synthetic_data(num_samples=5000)
        samples = samples[:10000]

        if not samples:
            return RetrainResult(
                status="error",
                old_reward=old_reward,
                new_reward=0.0,
                improvement_pct=0.0,
                epochs_trained=0,
                samples_used=0,
                checkpoint_path="",
                onnx_path=None,
                timestamp=timestamp,
                details="No training data available",
            )

        # Split: 80% train, 20% eval
        split = int(len(samples) * 0.8)
        train_samples = samples[:split]
        eval_samples = samples[split:]

        # Train
        trained_model, new_reward, epochs = self.train_model(current_model, train_samples)

        # Evaluate
        old_q = self.evaluate_model(current_model, eval_samples)
        new_q = self.evaluate_model(trained_model, eval_samples)

        improvement_pct = ((new_q - old_q) / abs(old_q) * 100.0) if old_q != 0 else 0.0
        promote = new_q >= old_q * (1.0 + self.promote_threshold)

        if promote:
            ckpt_path = self.save_checkpoint(trained_model, new_reward, "promoted")
            onnx_path = self.export_onnx(
                trained_model,
                "triton_model_repository/6hit_dqn/1/model.onnx",
            )
            status = "promoted"
            details = f"Model promoted: Q improved {improvement_pct:+.1f}%"
        else:
            ckpt_path = self.save_checkpoint(trained_model, new_reward, "rejected")
            onnx_path = None
            status = "rejected"
            details = f"Model rejected: Q improved only {improvement_pct:+.1f}% (need {self.promote_threshold*100:.0f}%)"

        # Log result
        result = RetrainResult(
            status=status,
            old_reward=old_reward,
            new_reward=new_reward,
            improvement_pct=improvement_pct,
            epochs_trained=epochs,
            samples_used=len(train_samples),
            checkpoint_path=ckpt_path,
            onnx_path=onnx_path,
            timestamp=timestamp,
            details=details,
        )

        log_path = os.path.join(self.checkpoint_dir, "retrain_log.jsonl")
        with open(log_path, "a") as f:
            f.write(json.dumps({
                "status": result.status,
                "old_reward": result.old_reward,
                "new_reward": result.new_reward,
                "improvement_pct": result.improvement_pct,
                "epochs_trained": result.epochs_trained,
                "samples_used": result.samples_used,
                "checkpoint_path": result.checkpoint_path,
                "onnx_path": result.onnx_path,
                "timestamp": result.timestamp,
                "details": result.details,
            }) + "\n")

        return result


# ─────────────────────────────────────────────────────────
# CLI Entry Point
# ─────────────────────────────────────────────────────────

if __name__ == "__main__":
    import sys

    print("=" * 60)
    print("AEGIS Auto-Retraining Pipeline")
    print("=" * 60)
    print()

    retrainer = AutoRetrainer(
        checkpoint_dir="/tmp/aegis/checkpoints",
        training_data_dir="/tmp/aegis/training_data",
        promote_threshold=0.05,
        max_epochs=20,
    )

    result = retrainer.run()

    print(f"Status:        {result.status}")
    print(f"Old reward:    {result.old_reward:.4f}")
    print(f"New reward:    {result.new_reward:.4f}")
    print(f"Improvement:   {result.improvement_pct:+.1f}%")
    print(f"Epochs:        {result.epochs_trained}")
    print(f"Samples:       {result.samples_used}")
    print(f"Checkpoint:    {result.checkpoint_path}")
    if result.onnx_path:
        print(f"ONNX:          {result.onnx_path}")
    print(f"Details:       {result.details}")
    print()
    print(f"Timestamp: {result.timestamp}")
