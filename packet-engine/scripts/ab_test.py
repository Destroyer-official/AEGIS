#!/usr/bin/env python3
"""
A/B Test Script for Model Registry.

Simulates canary routing between primary (v1) and canary (v2) model versions,
tracks false positive rates, and triggers automatic rollback when threshold is exceeded.

Usage:
    python ab_test.py --model refusal-vector --primary 1.0 --canary 2.0 --requests 1000
"""

import argparse
import hashlib
import json
import random
import sys
import time
from dataclasses import dataclass, field
from typing import List, Optional


@dataclass
class CanaryConfig:
    """Canary routing configuration."""
    canary_weight: float = 0.1  # 10% traffic to canary
    rollback_threshold: float = 0.05  # 5% FP rate triggers rollback
    min_observations: int = 100
    evaluation_window_seconds: int = 3600  # 1 hour


@dataclass
class Observation:
    """A single prediction observation."""
    true_positive: bool
    false_positive: bool
    confidence: float
    timestamp: float = field(default_factory=time.time)


@dataclass
class ModelVersion:
    """Model version in the registry."""
    name: str
    version: str
    status: str = "staging"
    observations: List[Observation] = field(default_factory=list)


class ModelRegistry:
    """MLflow-compatible model registry with canary routing."""

    def __init__(self):
        self.models = {}
        self.canary_configs = {}
        self.active_canary = {}

    def register_model(self, name: str, version: str):
        """Register a new model version."""
        if name not in self.models:
            self.models[name] = {}
        
        if version in self.models[name]:
            raise ValueError(f"Version {version} already registered for {name}")
        
        self.models[name][version] = ModelVersion(name=name, version=version)

    def set_status(self, name: str, version: str, status: str):
        """Set model version status."""
        if name not in self.models or version not in self.models[name]:
            raise ValueError(f"Model {name} version {version} not found")
        
        self.models[name][version].status = status

    def configure_canary(self, name: str, canary_version: str, config: CanaryConfig):
        """Configure canary routing for a model."""
        if name not in self.models or canary_version not in self.models[name]:
            raise ValueError(f"Model {name} version {canary_version} not found")
        
        if config.canary_weight < 0 or config.canary_weight > 1:
            raise ValueError(f"Invalid canary weight: {config.canary_weight}")
        
        self.canary_configs[name] = config
        self.active_canary[name] = canary_version
        self.set_status(name, canary_version, "canary")

    def route(self, name: str, request_id: int) -> str:
        """Make a routing decision for a request."""
        if name not in self.canary_configs:
            return "primary"
        
        config = self.canary_configs[name]
        # Deterministic routing based on request ID
        bucket = (request_id % 1000) / 1000.0
        
        if bucket < config.canary_weight:
            return "canary"
        return "primary"

    def record_observation(self, name: str, version: str, observation: Observation):
        """Record a prediction observation."""
        if name not in self.models or version not in self.models[name]:
            raise ValueError(f"Model {name} version {version} not found")
        
        self.models[name][version].observations.append(observation)

    def compute_fp_rate(self, name: str, version: str) -> float:
        """Compute false positive rate within evaluation window."""
        if name not in self.models or version not in self.models[name]:
            raise ValueError(f"Model {name} version {version} not found")
        
        if name not in self.canary_configs:
            return 0.0
        
        config = self.canary_configs[name]
        cutoff = time.time() - config.evaluation_window_seconds
        
        recent = [
            obs for obs in self.models[name][version].observations
            if obs.timestamp >= cutoff
        ]
        
        if len(recent) < config.min_observations:
            return 0.0
        
        fp_count = sum(1 for obs in recent if obs.false_positive)
        return fp_count / len(recent)

    def check_rollback(self, name: str) -> Optional[str]:
        """Check if rollback should trigger for the canary version."""
        if name not in self.active_canary:
            return None
        
        canary_version = self.active_canary[name]
        config = self.canary_configs.get(name)
        
        if not config:
            return None
        
        fp_rate = self.compute_fp_rate(name, canary_version)
        observations = len(self.models[name][canary_version].observations)
        
        if observations >= config.min_observations and fp_rate > config.rollback_threshold:
            return canary_version
        
        return None

    def execute_rollback(self, name: str) -> str:
        """Execute rollback: archive canary, return message."""
        if name not in self.active_canary:
            return f"No active canary for {name}"
        
        canary_version = self.active_canary[name]
        self.set_status(name, canary_version, "rolled_back")
        del self.active_canary[name]
        
        return f"Rolled back {name} version {canary_version}"


def simulate_prediction(version: str, is_adversarial: bool = False) -> Observation:
    """Simulate a model prediction."""
    # Simulate different FP rates for different versions
    if version == "1.0":
        # v1: 2% FP rate
        fp = random.random() < 0.02
        tp = not fp and is_adversarial
        confidence = random.uniform(0.7, 0.95)
    else:
        # v2: 8% FP rate (deliberately worse to trigger rollback)
        fp = random.random() < 0.08
        tp = not fp and is_adversarial
        confidence = random.uniform(0.6, 0.9)
    
    return Observation(
        true_positive=tp,
        false_positive=fp,
        confidence=confidence,
    )


def run_ab_test(args):
    """Run the A/B test simulation."""
    print(f"Starting A/B test for model: {args.model}")
    print(f"Primary: v{args.primary}, Canary: v{args.canary}")
    print(f"Canary weight: {args.canary_weight:.1%}")
    print(f"Rollback threshold: {args.rollback_threshold:.1%}")
    print("-" * 60)
    
    # Set up registry
    registry = ModelRegistry()
    registry.register_model(args.model, args.primary)
    registry.register_model(args.model, args.canary)
    registry.set_status(args.model, args.primary, "active")
    
    config = CanaryConfig(
        canary_weight=args.canary_weight,
        rollback_threshold=args.rollback_threshold,
        min_observations=args.min_observations,
    )
    registry.configure_canary(args.model, args.canary, config)
    
    # Run test
    primary_count = 0
    canary_count = 0
    rollback_triggered = False
    
    for i in range(args.requests):
        # Route request
        destination = registry.route(args.model, i)
        
        # Simulate adversarial vs benign traffic (10% adversarial)
        is_adversarial = random.random() < 0.1
        
        if destination == "primary":
            observation = simulate_prediction(args.primary, is_adversarial)
            registry.record_observation(args.model, args.primary, observation)
            primary_count += 1
        else:
            observation = simulate_prediction(args.canary, is_adversarial)
            registry.record_observation(args.model, args.canary, observation)
            canary_count += 1
        
        # Check for rollback every 100 requests
        if (i + 1) % 100 == 0:
            rollback_version = registry.check_rollback(args.model)
            if rollback_version:
                print(f"\n*** ROLLBACK TRIGGERED at request {i + 1} ***")
                print(f"Canary v{rollback_version} exceeded FP threshold")
                result = registry.execute_rollback(args.model)
                print(f"Result: {result}")
                rollback_triggered = True
                break
            
            # Print progress
            primary_fp = registry.compute_fp_rate(args.model, args.primary)
            canary_fp = registry.compute_fp_rate(args.model, args.canary)
            print(f"Request {i + 1}/{args.requests}: "
                  f"primary={primary_count}, canary={canary_count}, "
                  f"primary_fp={primary_fp:.3f}, canary_fp={canary_fp:.3f}")
    
    # Final summary
    print("\n" + "=" * 60)
    print("A/B Test Summary")
    print("=" * 60)
    
    summary = registry.models[args.model]
    for version, model in summary.items():
        fp_rate = registry.compute_fp_rate(args.model, version)
        obs_count = len(model.observations)
        fp_count = sum(1 for o in model.observations if o.false_positive)
        print(f"  v{version}: {obs_count} observations, {fp_count} FP ({fp_rate:.3f})")
    
    print(f"\nPrimary requests: {primary_count}")
    print(f"Canary requests: {canary_count}")
    print(f"Rollback triggered: {rollback_triggered}")
    
    # Export results
    if args.output:
        results = {
            "model": args.model,
            "primary_version": args.primary,
            "canary_version": args.canary,
            "canary_weight": args.canary_weight,
            "rollback_threshold": args.rollback_threshold,
            "total_requests": args.requests,
            "primary_requests": primary_count,
            "canary_requests": canary_count,
            "rollback_triggered": rollback_triggered,
            "final_fp_rates": {
                v: registry.compute_fp_rate(args.model, v)
                for v in [args.primary, args.canary]
            },
        }
        
        with open(args.output, "w") as f:
            json.dump(results, f, indent=2)
        print(f"\nResults exported to {args.output}")
    
    return 0 if not rollback_triggered else 1


def main():
    parser = argparse.ArgumentParser(
        description="A/B Test Script for Model Registry"
    )
    parser.add_argument("--model", required=True, help="Model name")
    parser.add_argument("--primary", required=True, help="Primary version (e.g., 1.0)")
    parser.add_argument("--canary", required=True, help="Canary version (e.g., 2.0)")
    parser.add_argument("--requests", type=int, default=1000, help="Number of requests to simulate")
    parser.add_argument("--canary-weight", type=float, default=0.1, help="Canary traffic weight (0.0-1.0)")
    parser.add_argument("--rollback-threshold", type=float, default=0.05, help="FP rate threshold for rollback")
    parser.add_argument("--min-observations", type=int, default=100, help="Min observations before rollback check")
    parser.add_argument("--output", help="Output JSON file path")
    parser.add_argument("--seed", type=int, help="Random seed for reproducibility")
    
    args = parser.parse_args()
    
    if args.seed is not None:
        random.seed(args.seed)
    
    sys.exit(run_ab_test(args))


if __name__ == "__main__":
    main()
