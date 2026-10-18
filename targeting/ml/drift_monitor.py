"""Model Drift Monitor — 6Hit DQN Inference Log Analysis

§3.6.1: "Monitoring model performance and data drift for continuous improvement."

Monitors Triton inference logs for feature distribution drift using
KL-divergence and alerts when drift exceeds threshold.

Zero external dependencies: reads local Triton logs, uses numpy for stats.
"""

from __future__ import annotations

import json
import math
import os
import time
from collections import deque
from dataclasses import dataclass, field

# ─────────────────────────────────────────────────────────
# Feature Statistics (baseline from training data)
# ─────────────────────────────────────────────────────────

# Baseline feature means (from training set of 6Hit DQN)
BASELINE_MEANS = {
    "prefix_len": 48.0,
    "hit_rate": 0.5,
    "as_path_len": 3.0,
    "origin_asn": 12345.0,
    "address_density": 0.7,
    "alloc_granularity": 0.5,  # Slash64=0, Slash120=1, Unknown=0.5
    "probe_budget_remaining": 5000.0,
    "seed_count": 42.0,
    "expected_reward": 10.0,
}

BASELINE_STD = {
    "prefix_len": 8.0,
    "hit_rate": 0.2,
    "as_path_len": 1.5,
    "origin_asn": 5000.0,
    "address_density": 0.15,
    "alloc_granularity": 0.4,
    "probe_budget_remaining": 2000.0,
    "seed_count": 20.0,
    "expected_reward": 5.0,
}


# ─────────────────────────────────────────────────────────
# KL-Divergence Computation
# ─────────────────────────────────────────────────────────

def _histogram(values: list[float], bins: int = 30, range_min: float = -3.0, range_max: float = 3.0) -> list[float]:
    """Compute normalized histogram (probability distribution) from values."""
    bin_width = (range_max - range_min) / bins
    counts = [0.0] * bins
    for v in values:
        idx = int((v - range_min) / bin_width)
        idx = max(0, min(bins - 1, idx))
        counts[idx] += 1.0
    total = sum(counts)
    if total > 0:
        counts = [c / total for c in counts]
    else:
        counts = [1.0 / bins] * bins
    return counts


def kl_divergence(p: list[float], q: list[float], epsilon: float = 1e-10) -> float:
    """Compute KL(P || Q) between two probability distributions."""
    assert len(p) == len(q), "Distributions must have same length"
    kl = 0.0
    for pi, qi in zip(p, q):
        pi = max(pi, epsilon)
        qi = max(qi, epsilon)
        kl += pi * math.log(pi / qi)
    return kl


def jensen_shannon_divergence(p: list[float], q: list[float]) -> float:
    """Compute Jensen-Shannon divergence (symmetric KL variant)."""
    m = [(pi + qi) / 2.0 for pi, qi in zip(p, q)]
    return (kl_divergence(p, m) + kl_divergence(q, m)) / 2.0


# ─────────────────────────────────────────────────────────
# Drift Monitor
# ─────────────────────────────────────────────────────────

@dataclass
class DriftAlert:
    """Alert when feature distribution drifts beyond threshold."""
    feature: str
    kl_divergence: float
    threshold: float
    baseline_mean: float
    observed_mean: float
    sample_count: int
    timestamp: float = field(default_factory=time.time)

    @property
    def severity(self) -> str:
        ratio = self.kl_divergence / self.threshold
        if ratio > 2.0:
            return "CRITICAL"
        elif ratio > 1.5:
            return "HIGH"
        elif ratio > 1.0:
            return "MEDIUM"
        return "LOW"

    def to_dict(self) -> dict:
        return {
            "feature": self.feature,
            "kl_divergence": round(self.kl_divergence, 6),
            "threshold": self.threshold,
            "severity": self.severity,
            "baseline_mean": round(self.baseline_mean, 4),
            "observed_mean": round(self.observed_mean, 4),
            "sample_count": self.sample_count,
            "timestamp": self.timestamp,
        }


class DriftMonitor:
    """Monitor feature distribution drift in Triton inference logs.

    Tracks rolling window of recent predictions and compares feature
    distributions to training baseline using KL-divergence.

    Usage:
        monitor = DriftMonitor(window_size=1000, kl_threshold=0.1)
        monitor.ingest({"prefix_len": 48, "hit_rate": 0.5, ...})
        alerts = monitor.check_drift()
        if alerts:
            for alert in alerts:
                print(f"DRIFT: {alert.feature} KL={alert.kl_divergence:.4f}")
    """

    def __init__(
        self,
        window_size: int = 1000,
        kl_threshold: float = 0.1,
        check_interval: int = 100,
        alert_log_path: str | None = None,
    ):
        self.window_size = window_size
        self.kl_threshold = kl_threshold
        self.check_interval = check_interval
        self.alert_log_path = alert_log_path

        self.feature_buffers: dict[str, deque] = {}
        self.total_ingested = 0
        self.last_check = 0
        self.alerts: list[DriftAlert] = []

        for key in BASELINE_MEANS:
            self.feature_buffers[key] = deque(maxlen=window_size)

    def ingest(self, features: dict) -> None:
        """Infer features from a Triton inference log entry."""
        alloc_map = {"Host": 0.0, "Slash120": 0.25, "Slash64": 0.5, "Slash48": 0.75, "Slash32": 1.0, "Unknown": 0.5}
        for key in BASELINE_MEANS:
            if key in features:
                val = features[key]
                if isinstance(val, str):
                    val = alloc_map.get(val, 0.5)
                self.feature_buffers[key].append(float(val))
        self.total_ingested += 1

    def _normalize(self, feature: str, values: list[float]) -> list[float]:
        """Z-score normalize values against baseline."""
        mean = BASELINE_MEANS[feature]
        std = BASELINE_STD[feature]
        if std == 0:
            return [0.0] * len(values)
        return [(v - mean) / std for v in values]

    def check_drift(self) -> list[DriftAlert]:
        """Check all features for drift; return alerts if KL > threshold."""
        new_alerts = []
        for feature, buffer in self.feature_buffers.items():
            if len(buffer) < 30:
                continue
            values = list(buffer)
            normalized = self._normalize(feature, values)

            observed_dist = _histogram(normalized)
            baseline_dist = _histogram([0.0] * len(normalized))

            jsd = jensen_shannon_divergence(observed_dist, baseline_dist)
            observed_mean = sum(values) / len(values)

            if jsd > self.kl_threshold:
                alert = DriftAlert(
                    feature=feature,
                    kl_divergence=jsd,
                    threshold=self.kl_threshold,
                    baseline_mean=BASELINE_MEANS[feature],
                    observed_mean=observed_mean,
                    sample_count=len(values),
                )
                new_alerts.append(alert)
                self.alerts.append(alert)

        if new_alerts and self.alert_log_path:
            os.makedirs(os.path.dirname(self.alert_log_path) or ".", exist_ok=True)
            with open(self.alert_log_path, "a") as f:
                for alert in new_alerts:
                    f.write(json.dumps(alert.to_dict()) + "\n")

        return new_alerts

    def get_status(self) -> dict:
        """Get current monitor status."""
        feature_stats = {}
        for feature, buffer in self.feature_buffers.items():
            if len(buffer) > 0:
                values = list(buffer)
                feature_stats[feature] = {
                    "count": len(values),
                    "mean": round(sum(values) / len(values), 4),
                    "min": round(min(values), 4),
                    "max": round(max(values), 4),
                }
            else:
                feature_stats[feature] = {"count": 0, "mean": 0, "min": 0, "max": 0}

        return {
            "total_ingested": self.total_ingested,
            "window_size": self.window_size,
            "kl_threshold": self.kl_threshold,
            "total_alerts": len(self.alerts),
            "critical_alerts": sum(1 for a in self.alerts if a.severity == "CRITICAL"),
            "feature_stats": feature_stats,
        }

    def reset(self) -> None:
        """Reset all buffers and alerts."""
        for buffer in self.feature_buffers.values():
            buffer.clear()
        self.total_ingested = 0
        self.alerts.clear()


# ─────────────────────────────────────────────────────────
# Log Parser for Triton Inference Logs
# ─────────────────────────────────────────────────────────

def parse_triton_log(log_path: str) -> list[dict]:
    """Parse Triton inference log file into feature dictionaries.

    Expected format (JSONL):
        {"prefix_len": 48, "hit_rate": 0.5, "as_path_len": 3, ...}
    """
    entries = []
    if not os.path.exists(log_path):
        return entries

    with open(log_path, "r") as f:
        for line in f:
            line = line.strip()
            if not line:
                continue
            try:
                entry = json.loads(line)
                entries.append(entry)
            except json.JSONDecodeError:
                continue
    return entries


# ─────────────────────────────────────────────────────────
# CLI Entry Point
# ─────────────────────────────────────────────────────────

if __name__ == "__main__":
    import sys

    log_path = sys.argv[1] if len(sys.argv) > 1 else "/tmp/aegis/triton_inference.jsonl"
    window = int(sys.argv[2]) if len(sys.argv) > 2 else 1000
    threshold = float(sys.argv[3]) if len(sys.argv) > 3 else 0.1

    print("Drift Monitor")
    print(f"  Log path:   {log_path}")
    print(f"  Window:     {window}")
    print(f"  KL thresh:  {threshold}")
    print()

    monitor = DriftMonitor(
        window_size=window,
        kl_threshold=threshold,
        alert_log_path="/tmp/aegis/drift_alerts.jsonl",
    )

    entries = parse_triton_log(log_path)
    print(f"Parsed {len(entries)} log entries")

    for entry in entries:
        monitor.ingest(entry)

    alerts = monitor.check_drift()
    if alerts:
        print(f"\n{len(alerts)} DRIFT ALERTS:")
        for alert in alerts:
            print(f"  [{alert.severity}] {alert.feature}: KL={alert.kl_divergence:.4f} "
                  f"(baseline={alert.baseline_mean:.2f}, observed={alert.observed_mean:.2f})")
    else:
        print("\nNo drift detected.")

    status = monitor.get_status()
    print(f"\nStatus: {status['total_ingested']} samples, {status['total_alerts']} alerts")
