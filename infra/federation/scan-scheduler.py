#!/usr/bin/env python3
"""AEGIS Federated Scan Scheduler - schedules scans to best cluster"""

import os, sys, json, time, logging
from datetime import datetime, timezone
from typing import Dict, List, Optional

try:
    from kubernetes import client, config
    from kubernetes.client.rest import ApiException
except ImportError:
    os.system(f"{sys.executable} -m pip install kubernetes -q")
    from kubernetes import client, config
    from kubernetes.client.rest import ApiException

logging.basicConfig(level=logging.INFO, format='%(asctime)s [%(levelname)s] %(message)s')
logger = logging.getLogger(__name__)

class ScanTarget:
    def __init__(self, ip: str, port: int = 443, region: str = ""):
        self.ip = ip
        self.port = port
        self.region = region

REGION_LATENCY = {
    "us-east-1": 10, "us-east-2": 25, "us-west-1": 50, "us-west-2": 60,
    "eu-west-1": 100, "eu-central-1": 110, "ap-southeast-1": 150,
    "ap-northeast-1": 160}

class FederatedScanScheduler:
    def __init__(self):
        try:
            config.load_incluster_config()
        except config.ConfigException:
            config.load_kube_config()
        self.custom_api = client.CustomObjectsApi()
        self.group = "aegis.io"
        self.version = "v1alpha1"
        self.plural = "clusterregistries"
        self.round_robin_index = 0

    def get_cluster_registry(self) -> Optional[Dict]:
        try:
            result = self.custom_api.list_cluster_custom_object(
                group=self.group, version=self.version, plural=self.plural)
            items = result.get("items", [])
            return items[0] if items else None
        except ApiException as e:
            logger.error(f"Failed to get ClusterRegistry: {e}")
            return None

    def get_ready_clusters(self, registry: Dict) -> List[Dict]:
        spec = registry.get("spec", {})
        status = registry.get("status", {})
        clusters = spec.get("clusters", [])
        cluster_status = {s["name"]: s for s in status.get("clusterStatus", [])}
        ready = []
        for cluster in clusters:
            cs = cluster_status.get(cluster["name"], {})
            if cs.get("ready", False):
                cluster["latencyMs"] = cs.get("latencyMs", 999)
                cluster["load"] = cs.get("load", 0.5)
                ready.append(cluster)
        return ready

    def schedule_closest(self, clusters: List[Dict], target: ScanTarget) -> Optional[Dict]:
        if not clusters:
            return None
        best, best_score = None, float("inf")
        for cluster in clusters:
            c_region = cluster.get("region", "us-east-1")
            latency = cluster.get("latencyMs", 999)
            t_penalty = REGION_LATENCY.get(target.region, 100) if target.region else 0
            c_penalty = REGION_LATENCY.get(c_region, 100)
            score = latency + abs(t_penalty - c_penalty)
            if score < best_score:
                best_score = score
                best = cluster
        return best

    def schedule_round_robin(self, clusters: List[Dict]) -> Optional[Dict]:
        if not clusters: return None
        self.round_robin_index = (self.round_robin_index + 1) % len(clusters)
        return clusters[self.round_robin_index]

    def schedule_least_loaded(self, clusters: List[Dict]) -> Optional[Dict]:
        return min(clusters, key=lambda c: c.get("load", 0.5)) if clusters else None

    def schedule_cost_optimized(self, clusters: List[Dict]) -> Optional[Dict]:
        if not clusters: return None
        def cost_score(c):
            base = 0.4 if "spot" in c.get("capabilities", []) else 1.0
            if "high-memory" in c.get("capabilities", []): base *= 1.5
            return base * (1 + c.get("load", 0.5))
        return min(clusters, key=cost_score)

    def schedule_scan(self, target: ScanTarget, strategy: str = "closest") -> Optional[Dict]:
        registry = self.get_cluster_registry()
        if not registry:
            return None
        spec = registry.get("spec", {})
        actual_strategy = spec.get("scanScheduling", {}).get("strategy", strategy)
        clusters = self.get_ready_clusters(registry)
        if not clusters:
            return None
        logger.info(f"Scheduling scan to {target.ip}:{target.port} ({target.region}) via {actual_strategy}")
        if actual_strategy == "closest": return self.schedule_closest(clusters, target)
        elif actual_strategy == "round-robin": return self.schedule_round_robin(clusters)
        elif actual_strategy == "least-loaded": return self.schedule_least_loaded(clusters)
        elif actual_strategy == "cost-optimized": return self.schedule_cost_optimized(clusters)
        return self.schedule_closest(clusters, target)

def main():
    logger.info("Starting Federated Scan Scheduler")
    scheduler = FederatedScanScheduler()
    while True:
        try:
            registry = scheduler.get_cluster_registry()
            if registry:
                clusters = scheduler.get_ready_clusters(registry)
                logger.info(f"Ready clusters: {len(clusters)}")
                for c in clusters:
                    logger.info(f"  - {c['name']} ({c.get('region', '?')}): latency={c.get('latencyMs', '?')}ms load={c.get('load', '?')}")
        except Exception as e:
            logger.error(f"Error: {e}")
        time.sleep(60)

if __name__ == "__main__":
    main()