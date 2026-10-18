"""Comprehensive tests for AEGIS ML targeting pipeline."""

from __future__ import annotations

import os

import torch

from auto_retrain import (
    TrainingSample,
    generate_synthetic_data,
)
from dqn_model import (
    ACTION_DIM,
    STATE_DIM,
    DQNAgent,
    DuelingDQN,
    encode_state,
)
from drift_monitor import (
    DriftMonitor,
    jensen_shannon_divergence,
    kl_divergence,
)
from export_onnx import create_triton_config, export_onnx


def test_dueling_dqn_forward():
    model = DuelingDQN(state_dim=STATE_DIM, action_dim=ACTION_DIM)
    dummy_input = torch.randn(4, STATE_DIM)
    q_vals = model(dummy_input)
    assert q_vals.shape == (4, ACTION_DIM)
    assert not torch.isnan(q_vals).any()


def test_encode_state():
    encoded = encode_state(
        prefix_len=48,
        hit_rate=0.45,
        as_path_len=3,
        origin_asn=13335,
        address_density=0.8,
        alloc_granularity="Slash64",
        probe_budget_remaining=8000,
        seed_count=50,
        expected_reward=12.0,
    )
    assert isinstance(encoded, torch.Tensor)
    assert encoded.shape == (STATE_DIM,)
    assert encoded.dtype == torch.float32


def test_dqn_agent_transitions_and_training():
    agent = DQNAgent(state_dim=STATE_DIM, action_dim=ACTION_DIM, batch_size=8)
    state = torch.randn(STATE_DIM)
    action = agent.act(state, epsilon=0.0)
    assert 0 <= action < ACTION_DIM

    # Replay buffer operations
    for _ in range(12):
        s = torch.randn(STATE_DIM)
        ns = torch.randn(STATE_DIM)
        agent.store_transition(s, action, 1.0, ns, False)

    assert len(agent.buffer) == 12
    loss = agent.train_step()
    assert loss is not None
    assert loss >= 0.0


def test_drift_divergence_metrics():
    dist_p = [0.1, 0.2, 0.4, 0.2, 0.1]
    dist_q = [0.1, 0.2, 0.4, 0.2, 0.1]
    # Identical distributions have 0 divergence
    assert abs(kl_divergence(dist_p, dist_q)) < 1e-6
    assert abs(jensen_shannon_divergence(dist_p, dist_q)) < 1e-6

    # Distinct distributions have positive divergence
    dist_alt = [0.5, 0.2, 0.1, 0.1, 0.1]
    assert kl_divergence(dist_p, dist_alt) > 0.0
    assert jensen_shannon_divergence(dist_p, dist_alt) > 0.0


def test_drift_monitor_alerting():
    monitor = DriftMonitor(window_size=50, kl_threshold=0.05, check_interval=10)
    # Ingest baseline-similar samples
    for _ in range(50):
        monitor.ingest({
            "prefix_len": 48.0,
            "hit_rate": 0.5,
            "as_path_len": 3.0,
            "origin_asn": 12345.0,
            "address_density": 0.7,
            "alloc_granularity": "Slash64",
            "probe_budget_remaining": 5000.0,
            "seed_count": 42.0,
            "expected_reward": 10.0,
        })
    alerts = monitor.check_drift()
    assert len(alerts) == 0

    # Ingest heavily drifted samples
    for _ in range(50):
        monitor.ingest({
            "prefix_len": 124.0,  # Extreme drift
            "hit_rate": 0.01,
            "as_path_len": 15.0,
        })
    alerts = monitor.check_drift()
    assert any(a.feature == "prefix_len" for a in alerts)


def test_synthetic_data_generation():
    samples = generate_synthetic_data(num_samples=20)
    assert len(samples) == 20
    first = samples[0]
    assert isinstance(first, TrainingSample)
    assert 0 <= first.action < ACTION_DIM


def test_onnx_export(tmp_path):
    import pytest

    output_path = tmp_path / "test_model.onnx"

    # 1. Must fail closed if checkpoint does not exist and allow_untrained is False
    with pytest.raises(FileNotFoundError):
        export_onnx(
            checkpoint_path="/nonexistent/dqn.pt",
            output_path=str(output_path),
            allow_untrained=False,
        )

    # 2. When allow_untrained=True is explicitly passed (e.g. test harness), verify ONNX architecture
    res_path = export_onnx(
        checkpoint_path="/nonexistent/dqn.pt",
        output_path=str(output_path),
        allow_untrained=True,
    )
    assert os.path.exists(res_path)
    assert os.path.getsize(res_path) > 1000

    config = create_triton_config(model_name="6hit_dqn_test")
    assert "6hit_dqn_test" in config
    assert "onnxruntime_onnx" in config
