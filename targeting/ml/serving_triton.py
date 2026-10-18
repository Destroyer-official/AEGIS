"""Triton Inference Server client for 6Hit DQN serving.

§3.6.1: Triton deployment — p99 <10ms, batch inference.
"""

from __future__ import annotations

import asyncio
import time

import numpy as np

try:
    import tritonclient.grpc as grpcclient
    import tritonclient.http as httpclient
    from tritonclient.utils import InferenceServerException
    HAS_TRITON_CLIENT = True
except ImportError:
    HAS_TRITON_CLIENT = False
    httpclient = None
    grpcclient = None
    InferenceServerException = None

from dqn_model import STATE_DIM, encode_state


class TritonDQNClient:
    """Async client for Triton Inference Server."""

    def __init__(
        self,
        url: str = "localhost:8000",
        model_name: str = "6hit_dqn",
        protocol: str = "http",
        verbose: bool = False,
    ):
        self.url = url
        self.model_name = model_name
        self.protocol = protocol

        if protocol == "http":
            self.client = httpclient.InferenceServerClient(url=url, verbose=verbose)
        elif protocol == "grpc":
            self.client = grpcclient.InferenceServerClient(url=url, verbose=verbose)
        else:
            raise ValueError(f"Unknown protocol: {protocol}")

    def is_ready(self) -> bool:
        """Check if server and model are ready."""
        try:
            return self.client.is_server_ready() and self.client.is_model_ready(self.model_name)
        except InferenceServerException:
            return False

    def predict(self, state: np.ndarray) -> np.ndarray:
        """Single inference request.

        Args:
            state: (STATE_DIM,) or (batch, STATE_DIM) float32 array

        Returns:
            Q-values: (batch, ACTION_DIM) float32 array
        """
        if state.ndim == 1:
            state = state.reshape(1, -1)

        inputs = [httpclient.InferInput("input", state.shape, "FP32")]
        inputs[0].set_data_from_numpy(state.astype(np.float32))

        outputs = [httpclient.InferRequestedOutput("output")]

        response = self.client.infer(self.model_name, inputs, outputs=outputs)
        return response.as_numpy("output")

    async def predict_async(self, state: np.ndarray) -> np.ndarray:
        """Async inference (HTTP only)."""
        loop = asyncio.get_event_loop()
        return await loop.run_in_executor(None, self.predict, state)

    def predict_batch(self, states: np.ndarray) -> np.ndarray:
        """Batch inference.

        Args:
            states: (batch, STATE_DIM) float32 array

        Returns:
            Q-values: (batch, ACTION_DIM) float32 array
        """
        return self.predict(states)

    def benchmark(self, num_requests: int = 1000, batch_size: int = 1) -> dict:
        """Benchmark latency and throughput."""
        # Warmup
        for _ in range(10):
            self.predict(np.random.randn(1, STATE_DIM).astype(np.float32))

        latencies = []
        start = time.perf_counter()

        for _ in range(num_requests):
            batch = np.random.randn(batch_size, STATE_DIM).astype(np.float32)
            t0 = time.perf_counter()
            _ = self.predict(batch)
            latencies.append((time.perf_counter() - t0) * 1000)

        total_time = time.perf_counter() - start

        latencies = np.array(latencies)
        return {
            "total_requests": num_requests,
            "batch_size": batch_size,
            "total_time_sec": total_time,
            "throughput_rps": num_requests / total_time,
            "latency_p50_ms": np.percentile(latencies, 50),
            "latency_p95_ms": np.percentile(latencies, 95),
            "latency_p99_ms": np.percentile(latencies, 99),
            "latency_max_ms": latencies.max(),
        }


class TritonDQNServing:
    """High-level serving wrapper matching FastAPI interface."""

    def __init__(self, triton_url: str = "localhost:8000", model_name: str = "6hit_dqn"):
        self.client = TritonDQNClient(triton_url, model_name)
        self.ACTION_NAMES = ["ProbeSlash64", "ProbeSlash120", "Skip", "Repartition"]

    def predict(self, state_dict: dict) -> dict:
        """Predict action from state dict (matches FastAPI interface)."""
        state_tensor = encode_state(
            prefix_len=state_dict["prefix_len"],
            hit_rate=state_dict["hit_rate"],
            as_path_len=state_dict["as_path_len"],
            origin_asn=state_dict["origin_asn"],
            address_density=state_dict["address_density"],
            alloc_granularity=state_dict["alloc_granularity"],
            probe_budget_remaining=state_dict["probe_budget_remaining"],
            seed_count=state_dict["seed_count"],
            expected_reward=state_dict["expected_reward"],
            region_coords=state_dict.get("region_coords", (0.0, 0.0, 0.0)),
        ).numpy()

        t0 = time.perf_counter()
        q_values = self.client.predict(state_tensor)
        latency_ms = (time.perf_counter() - t0) * 1000

        action_idx = int(q_values.argmax(axis=-1)[0])

        return {
            "action": self.ACTION_NAMES[action_idx],
            "action_idx": action_idx,
            "q_values": q_values[0].tolist(),
            "latency_ms": round(latency_ms, 3),
        }

    def health(self) -> dict:
        return {
            "status": "ok" if self.client.is_ready() else "unavailable",
            "model_loaded": self.client.is_ready(),
            "triton_url": self.client.url,
        }


# ─────────────────────────────────────────────────────────
# CLI
# ─────────────────────────────────────────────────────────

def main():
    import argparse

    parser = argparse.ArgumentParser(description="Triton DQN Client")
    parser.add_argument("--url", default="localhost:8000", help="Triton server URL")
    parser.add_argument("--model", default="6hit_dqn", help="Model name")
    parser.add_argument("--benchmark", action="store_true", help="Run benchmark")
    parser.add_argument("--requests", type=int, default=1000, help="Benchmark requests")
    parser.add_argument("--batch-size", type=int, default=1, help="Batch size")
    args = parser.parse_args()

    client = TritonDQNClient(args.url, args.model)

    if not client.is_ready():
        print(f"Triton server not ready at {args.url}")
        return

    print(f"Connected to Triton at {args.url}, model: {args.model}")

    if args.benchmark:
        print(f"Running benchmark: {args.requests} requests, batch_size={args.batch_size}")
        results = client.benchmark(args.requests, args.batch_size)
        print(f"  Throughput: {results['throughput_rps']:.1f} req/s")
        print(f"  Latency p50: {results['latency_p50_ms']:.2f} ms")
        print(f"  Latency p95: {results['latency_p95_ms']:.2f} ms")
        print(f"  Latency p99: {results['latency_p99_ms']:.2f} ms")
        print(f"  Latency max: {results['latency_max_ms']:.2f} ms")

        if results['latency_p99_ms'] < 10:
            print("  ✅ p99 < 10ms target MET")
        else:
            print("  ❌ p99 < 10ms target NOT MET")
    else:
        # Single test
        state = np.random.randn(1, STATE_DIM).astype(np.float32)
        q = client.predict(state)
        print(f"Test inference: Q-shape={q.shape}, values={q[0]}")


if __name__ == "__main__":
    main()