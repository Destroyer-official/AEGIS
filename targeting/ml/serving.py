"""Ray + FastAPI Serving Skeleton — 6Hit DQN Targeting

§3.6.1: "FastAPI microservice serving predictions with <100ms latency"
§3.6.2: "Ray for distributed training orchestration"

Target latency: <100ms per prediction request.
"""

from __future__ import annotations

import asyncio
import time
from contextlib import asynccontextmanager
from typing import Optional

import torch
from fastapi import FastAPI, HTTPException
from pydantic import BaseModel, Field

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
# Request / Response Models
# ─────────────────────────────────────────────────────────

class StateRequest(BaseModel):
    """Incoming state observation for action prediction."""
    prefix_len: int = Field(..., ge=0, le=128, description="IPv6 prefix length")
    hit_rate: float = Field(..., ge=0.0, le=1.0, description="Rolling hit rate")
    as_path_len: int = Field(..., ge=0, le=255, description="BGP AS-path length")
    origin_asn: int = Field(..., ge=0, description="Origin ASN")
    address_density: float = Field(..., ge=0.0, le=1.0)
    alloc_granularity: str = Field("Unknown", description="Allocation granularity")
    probe_budget_remaining: int = Field(..., ge=0)
    seed_count: int = Field(..., ge=0)
    expected_reward: float = Field(0.0)
    region_coords: tuple[float, float, float] = (0.0, 0.0, 0.0)


# ─────────────────────────────────────────────────────────
# Space Tree Visualization Models (§3.1.2)
# ─────────────────────────────────────────────────────────

class SpaceTreeNodeResponse(BaseModel):
    """Single node in the IPv6 space tree for visualization."""
    prefix: str = Field(..., description="IPv6 prefix (e.g., 2001:db8::/32)")
    prefix_len: int = Field(..., ge=0, le=128)
    seed_count: int = Field(..., ge=0, description="Number of seed addresses in this region")
    hit_rate: float = Field(..., ge=0.0, le=1.0, description="Historical hit rate")
    expected_reward: float = Field(..., description="Expected reward prior")
    saturated: bool = Field(..., description="Whether region is saturated")
    depth: int = Field(..., ge=0, description="Tree depth (0=root)")
    children: list["SpaceTreeNodeResponse"] = Field(default_factory=list, description="Child nodes")


class SpaceTreeRequest(BaseModel):
    """Request to build/return space tree."""
    root_prefix: str = Field("2001:db8::/32", description="Root IPv6 prefix")
    seeds: list[str] = Field(default_factory=list, description="Seed IPv6 addresses")
    max_depth: int = Field(4, ge=1, le=8, description="Maximum tree depth")
    split_threshold: int = Field(5, ge=1, description="Seed count threshold for splitting")


class SpaceTreeResponse(BaseModel):
    """Full space tree for visualization."""
    root: SpaceTreeNodeResponse
    total_nodes: int
    leaf_count: int
    max_depth: int


# Forward reference resolution
SpaceTreeNodeResponse.model_rebuild()


class ActionResponse(BaseModel):
    """Predicted action from the DQN agent."""
    action: str = Field(..., description="Action name: ProbeSlash64, ProbeSlash120, Skip, Repartition")
    action_idx: int = Field(..., description="Raw action index")
    q_values: list[float] = Field(..., description="Q-values for all actions")
    latency_ms: float = Field(..., description="Inference latency in milliseconds")


class TrainRequest(BaseModel):
    """Transition for online learning."""
    state: StateRequest
    action: int
    reward: float
    next_state: StateRequest
    done: bool = False


class TrainResponse(BaseModel):
    loss: Optional[float] = None
    buffer_size: int
    steps: int


class HealthResponse(BaseModel):
    status: str
    model_loaded: bool
    device: str
    uptime_seconds: float


# ─────────────────────────────────────────────────────────
# Global State
# ─────────────────────────────────────────────────────────

class AppState:
    agent: Optional[DQNAgent] = None
    device: str = "cpu"
    start_time: float = 0.0
    model_path: Optional[str] = None


state = AppState()
ACTION_NAMES = ["ProbeSlash64", "ProbeSlash120", "Skip", "Repartition"]


# ─────────────────────────────────────────────────────────
# FastAPI App
# ─────────────────────────────────────────────────────────

@asynccontextmanager
async def lifespan(app: FastAPI):
    """Load model on startup."""
    state.start_time = time.time()
    state.device = "cuda" if torch.cuda.is_available() else "cpu"

    state.agent = DQNAgent(state_dim=STATE_DIM, action_dim=ACTION_DIM)

    # Try loading a saved checkpoint
    import os
    model_path = os.environ.get("AEGIS_DQN_MODEL_PATH")
    if model_path and os.path.exists(model_path):
        state.agent.load(model_path)
        state.model_path = model_path

    state.agent.policy_net.to(state.device)
    state.agent.target_net.to(state.device)
    state.agent.policy_net.eval()

    yield

    # Shutdown
    pass


app = FastAPI(
    title="AEGIS 6Hit DQN Serving",
    description="§3.6.1 — RL IPv6 target traversal prediction service",
    version="0.1.0",
    lifespan=lifespan,
)


# ─────────────────────────────────────────────────────────
# Endpoints
# ─────────────────────────────────────────────────────────

@app.get("/health", response_model=HealthResponse)
async def health():
    return HealthResponse(
        status="ok",
        model_loaded=state.agent is not None,
        device=state.device,
        uptime_seconds=time.time() - state.start_time,
    )


@app.post("/predict", response_model=ActionResponse)
async def predict(req: StateRequest):
    """Predict the best action for a given state.

    Target latency: <100ms (including tensor transfer).
    """
    if state.agent is None:
        raise HTTPException(status_code=503, detail="Model not loaded")

    t0 = time.perf_counter()

    state_tensor = encode_state(
        prefix_len=req.prefix_len,
        hit_rate=req.hit_rate,
        as_path_len=req.as_path_len,
        origin_asn=req.origin_asn,
        address_density=req.address_density,
        alloc_granularity=req.alloc_granularity,
        probe_budget_remaining=req.probe_budget_remaining,
        seed_count=req.seed_count,
        expected_reward=req.expected_reward,
        region_coords=req.region_coords,
    ).to(state.device)

    with torch.no_grad():
        q_values = state.agent.policy_net(state_tensor.unsqueeze(0))
        action_idx = q_values.argmax(dim=-1).item()
        q_vals = q_values.squeeze(0).cpu().tolist()

    latency_ms = (time.perf_counter() - t0) * 1000

    return ActionResponse(
        action=ACTION_NAMES[action_idx],
        action_idx=action_idx,
        q_values=q_vals,
        latency_ms=round(latency_ms, 3),
    )


@app.post("/train", response_model=TrainResponse)
async def train(req: TrainRequest):
    """Online learning: store a transition and optionally train."""
    if state.agent is None:
        raise HTTPException(status_code=503, detail="Model not loaded")

    s = encode_state(
        prefix_len=req.state.prefix_len,
        hit_rate=req.state.hit_rate,
        as_path_len=req.state.as_path_len,
        origin_asn=req.state.origin_asn,
        address_density=req.state.address_density,
        alloc_granularity=req.state.alloc_granularity,
        probe_budget_remaining=req.state.probe_budget_remaining,
        seed_count=req.state.seed_count,
        expected_reward=req.state.expected_reward,
        region_coords=req.state.region_coords,
    )

    ns = encode_state(
        prefix_len=req.next_state.prefix_len,
        hit_rate=req.next_state.hit_rate,
        as_path_len=req.next_state.as_path_len,
        origin_asn=req.next_state.origin_asn,
        address_density=req.next_state.address_density,
        alloc_granularity=req.next_state.alloc_granularity,
        probe_budget_remaining=req.next_state.probe_budget_remaining,
        seed_count=req.next_state.seed_count,
        expected_reward=req.next_state.expected_reward,
        region_coords=req.next_state.region_coords,
    )

    state.agent.store_transition(s, req.action, req.reward, ns, req.done)
    loss = state.agent.train_step()

    return TrainResponse(
        loss=loss,
        buffer_size=len(state.agent.buffer),
        steps=state.agent.steps,
    )


@app.get("/model/info")
async def model_info():
    """Return model architecture summary."""
    if state.agent is None:
        raise HTTPException(status_code=503, detail="Model not loaded")

    policy = state.agent.policy_net
    total_params = sum(p.numel() for p in policy.parameters())
    trainable_params = sum(p.numel() for p in policy.parameters() if p.requires_grad)

    return {
        "architecture": "DuelingDQN",
        "state_dim": policy.state_dim,
        "action_dim": policy.action_dim,
        "total_params": total_params,
        "trainable_params": trainable_params,
        "device": state.device,
        "training_steps": state.agent.steps,
        "buffer_size": len(state.agent.buffer),
    }


# ─────────────────────────────────────────────────────────
# CLI Entry Point
# ─────────────────────────────────────────────────────────

def main():
    import uvicorn
    import os

    host = os.environ.get("AEGIS_SERVE_HOST", "0.0.0.0")
    port = int(os.environ.get("AEGIS_SERVE_PORT", "8000"))

    print(f"[AEGIS] Starting 6Hit DQN serving on {host}:{port}")
    uvicorn.run(app, host=host, port=port, log_level="info")


# ─────────────────────────────────────────────────────────
# Space Tree Visualization (§3.1.2)
# ─────────────────────────────────────────────────────────

import ipaddress
from typing import Optional

class _SpaceTreeNode:
    """Internal space tree node for building visualization."""
    def __init__(self, prefix: ipaddress.IPv6Network, seed_count: int = 0):
        self.prefix = prefix
        self.seed_count = seed_count
        self.hit_rate = 0.0
        self.expected_reward = float(seed_count)
        self.saturated = False
        self.children: list[_SpaceTreeNode] = []

    def split(self, k: int = 2):
        """Split this prefix into k sub-prefixes."""
        if self.prefix.prefixlen >= 124:
            return
        new_len = self.prefix.prefixlen + 1
        if new_len > 124:
            new_len = 124
        subnets = list(self.prefix.subnets(new_prefix=new_len))
        for subnet in subnets[:k]:
            self.children.append(_SpaceTreeNode(subnet, 0))

    def to_response(self, depth: int = 0) -> SpaceTreeNodeResponse:
        return SpaceTreeNodeResponse(
            prefix=str(self.prefix),
            prefix_len=self.prefix.prefixlen,
            seed_count=self.seed_count,
            hit_rate=self.hit_rate,
            expected_reward=self.expected_reward,
            saturated=self.saturated,
            depth=depth,
            children=[c.to_response(depth + 1) for c in self.children],
        )


def build_space_tree(
    seeds: list[str],
    root_prefix: str,
    max_depth: int,
    split_threshold: int,
) -> _SpaceTreeNode:
    """Build IPv6 space tree using divisive hierarchical clustering."""
    root_net = ipaddress.IPv6Network(root_prefix, strict=False)
    root = _SpaceTreeNode(root_net, len(seeds))

    # Insert seeds
    for seed_str in seeds:
        try:
            seed_addr = ipaddress.IPv6Address(seed_str)
            _insert_seed(root, seed_addr, split_threshold, max_depth, 0)
        except ValueError:
            continue

    return root


def _insert_seed(node: _SpaceTreeNode, seed: ipaddress.IPv6Address, threshold: int, max_depth: int, depth: int):
    if depth >= max_depth or node.prefix.prefixlen >= 124:
        node.seed_count += 1
        return

    if node.seed_count < threshold and node.children:
        # Already split, descend
        for child in node.children:
            if seed in child.prefix:
                _insert_seed(child, seed, threshold, max_depth, depth + 1)
                return
        return

    node.seed_count += 1

    if node.seed_count > threshold and not node.children:
        node.split(2)
        # Re-distribute (simplified: just mark for next insert)


def collect_stats(node: _SpaceTreeNode) -> tuple[int, int, int]:
    """Return (total_nodes, leaf_count, max_depth)."""
    total = 1
    leaves = 0
    max_d = 0
    if not node.children:
        leaves = 1
    else:
        for child in node.children:
            t, l, d = collect_stats(child)
            total += t
            leaves += l
            max_d = max(max_d, d + 1)
    return total, leaves, max_d


@app.get("/space-tree", response_model=SpaceTreeResponse)
async def get_space_tree(
    root_prefix: str = "2001:db8::/32",
    seeds: str = "",
    max_depth: int = 4,
    split_threshold: int = 5,
):
    """Return IPv6 space tree visualization data.

    §3.1.2: Space tree from divisive hierarchical clustering on seed addresses.
    Returns JSON with prefix clusters for dashboard visualization.
    """
    seed_list = [s.strip() for s in seeds.split(",") if s.strip()] if seeds else []

    # Generate demo seeds if none provided
    if not seed_list:
        import random
        base = ipaddress.IPv6Network(root_prefix, strict=False)
        seed_list = [str(base[random.randint(1, int(base.num_addresses) - 2)]) for _ in range(50)]

    tree = build_space_tree(seed_list, root_prefix, max_depth, split_threshold)
    total, leaves, max_d = collect_stats(tree)

    return SpaceTreeResponse(
        root=tree.to_response(),
        total_nodes=total,
        leaf_count=leaves,
        max_depth=max_d,
    )


if __name__ == "__main__":
    main()
