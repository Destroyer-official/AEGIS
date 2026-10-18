#!/usr/bin/env python3
"""AEGIS Cost Optimization Controller"""

import os, sys, json, time, logging
from datetime import datetime, timezone
from typing import Dict, List, Optional
from dataclasses import dataclass, asdict

try:
    from kubernetes import client, config
    from kubernetes.client.rest import ApiException
except ImportError:
    os.system(f"{sys.executable} -m pip install kubernetes -q")
    from kubernetes import client, config
    from kubernetes.client.rest import ApiException

logging.basicConfig(level=logging.INFO, format='%(asctime)s [%(levelname)s] %(message)s')
logger = logging.getLogger(__name__)

@dataclass
class CostRecommendation:
    resource_type: str
    resource_name: str
    namespace: str
    recommendation_type: str
    current_value: str
    recommended_value: str
    estimated_savings_percent: float
    description: str
    priority: str

@dataclass
class CostReport:
    timestamp: str
    total_recommendations: int
    estimated_monthly_savings_usd: float
    recommendations: List[Dict]
    summary: Dict

class AEGISCostController:
    def __init__(self):
        try:
            config.load_incluster_config()
        except config.ConfigException:
            config.load_kube_config()
        self.core_v1 = client.CoreV1Api()
        self.custom_api = client.CustomObjectsApi()
        self.target_namespaces = ["default", "monitoring"]
        self.cost_data_dir = os.environ.get("COST_DATA_DIR", "/var/lib/cost-controller")
        self.report_interval = int(os.environ.get("REPORT_INTERVAL", "300"))

    def get_pod_metrics(self, namespace: str) -> Dict:
        try:
            metrics = self.custom_api.list_namespaced_custom_object(
                group="metrics.k8s.io", version="v1beta1",
                namespace=namespace, plural="pods"
            )
            return {item["metadata"]["name"]: item for item in metrics.get("items", [])}
        except ApiException as e:
            logger.warning(f"Could not get pod metrics for {namespace}: {e}")
            return {}

    def get_node_metrics(self) -> Dict:
        try:
            metrics = self.custom_api.list_cluster_custom_object(
                group="metrics.k8s.io", version="v1beta1", plural="nodes"
            )
            return {item["metadata"]["name"]: item for item in metrics.get("items", [])}
        except ApiException as e:
            logger.warning(f"Could not get node metrics: {e}")
            return {}

    def analyze_pod_resources(self, namespace: str) -> List[CostRecommendation]:
        recommendations = []
        pods = self.core_v1.list_namespaced_pod(namespace)
        metrics = self.get_pod_metrics(namespace)
        for pod in pods.items:
            if pod.status.phase != "Running":
                continue
            for container in pod.spec.containers:
                if not container.resources:
                    continue
                req = container.resources.requests or {}
                req_cpu = self._parse_cpu(req.get("cpu", "100m"))
                req_mem = self._parse_memory(req.get("memory", "128Mi"))
                pod_m = metrics.get(pod.metadata.name, {})
                cm = next((c for c in pod_m.get("containers", []) if c["name"] == container.name), {})
                if not cm:
                    continue
                actual_cpu = self._parse_cpu(cm.get("usage", {}).get("cpu", "0"))
                actual_mem = self._parse_memory(cm.get("usage", {}).get("memory", "0"))
                if req_cpu > 0 and actual_cpu < req_cpu * 0.3:
                    rec_cpu = max(int(actual_cpu * 2), 50)
                    savings = (1 - rec_cpu / req_cpu) * 100
                    recommendations.append(CostRecommendation(
                        "Pod", f"{pod.metadata.name}/{container.name}", namespace,
                        "CPU Right-sizing", f"{req_cpu}m", f"{rec_cpu}m",
                        round(savings, 1),
                        f"CPU usage {actual_cpu}m, requesting {req_cpu}m ({actual_cpu/req_cpu*100:.0f}%)",
                        "medium"))
                if req_mem > 0 and actual_mem < req_mem * 0.3:
                    rec_mem = max(int(actual_mem * 2), 64)
                    savings = (1 - rec_mem / req_mem) * 100
                    recommendations.append(CostRecommendation(
                        "Pod", f"{pod.metadata.name}/{container.name}", namespace,
                        "Memory Right-sizing", f"{req_mem}Mi", f"{rec_mem}Mi",
                        round(savings, 1),
                        f"Memory usage {actual_mem}Mi, requesting {req_mem}Mi ({actual_mem/req_mem*100:.0f}%)",
                        "medium"))
        return recommendations

    def analyze_pvcs(self, namespace: str) -> List[CostRecommendation]:
        recommendations = []
        pvcs = self.core_v1.list_namespaced_persistent_volume_claim(namespace)
        pods = self.core_v1.list_namespaced_pod(namespace)
        pvc_usage = {}
        for pod in pods.items:
            if pod.status.phase != "Running":
                continue
            for vol in (pod.spec.volumes or []):
                if vol.persistent_volume_claim:
                    name = vol.persistent_volume_claim.claim_name
                    pvc_usage[name] = pvc_usage.get(name, 0) + 1
        for pvc in pvcs.items:
            if pvc.status.phase != "Bound":
                continue
            capacity = self._parse_memory(pvc.status.capacity.get("storage", "10Gi"))
            bound_pods = pvc_usage.get(pvc.metadata.name, 0)
            if capacity > 10240 and bound_pods == 0:
                recommendations.append(CostRecommendation(
                    "PVC", pvc.metadata.name, namespace,
                    "Idle PVC", f"{capacity // 1024}Gi", "Delete or downsize",
                    90.0, f"PVC bound but unused by any pod", "high"))
        return recommendations

    def analyze_idle_resources(self, namespace: str) -> List[CostRecommendation]:
        return []

    def analyze_spot_instances(self) -> List[CostRecommendation]:
        recommendations = []
        nodes = self.core_v1.list_node()
        for node in nodes.items:
            taints = node.spec.taints or []
            is_spot = any("spot" in str(t).lower() for t in taints)
            if not is_spot:
                recommendations.append(CostRecommendation(
                    "Node", node.metadata.name, "",
                    "Spot Instance", "On-demand", "Spot",
                    60.0, f"Node could use spot instances for ~60% savings", "low"))
        return recommendations

    def generate_report(self) -> CostReport:
        all_recs = []
        for ns in self.target_namespaces:
            all_recs.extend(self.analyze_pod_resources(ns))
            all_recs.extend(self.analyze_pvcs(ns))
        all_recs.extend(self.analyze_spot_instances())
        all_recs.sort(key=lambda r: ({"high": 0, "medium": 1, "low": 2}.get(r.priority, 3), -r.estimated_savings_percent))
        est = sum(r.estimated_savings_percent * 0.5 for r in all_recs if r.estimated_savings_percent > 0)
        summary = {"high": len([r for r in all_recs if r.priority == "high"]),
                    "medium": len([r for r in all_recs if r.priority == "medium"]),
                    "low": len([r for r in all_recs if r.priority == "low"]),
                    "by_type": {}}
        for r in all_recs:
            summary["by_type"][r.recommendation_type] = summary["by_type"].get(r.recommendation_type, 0) + 1
        return CostReport(
            timestamp=datetime.now(timezone.utc).isoformat(),
            total_recommendations=len(all_recs),
            estimated_monthly_savings_usd=round(est, 2),
            recommendations=[asdict(r) for r in all_recs],
            summary=summary)

    def _parse_cpu(self, value: str) -> int:
        if not value: return 0
        v = str(value)
        return int(v[:-1]) if v.endswith("m") else int(float(v) * 1000)

    def _parse_memory(self, value: str) -> int:
        if not value: return 0
        v = str(value)
        for s, m in {"Ki": 1, "Mi": 1024, "Gi": 1048576, "Ti": 1073741824}.items():
            if v.endswith(s): return int(v[:-len(s)]) * m
        return int(v)

def main():
    logger.info("Starting AEGIS Cost Optimization Controller")
    controller = AEGISCostController()
    while True:
        try:
            report = controller.generate_report()
            logger.info(f"Cost report: {report.total_recommendations} recs, est. ${report.estimated_monthly_savings_usd}/month")
            os.makedirs(os.path.join(controller.cost_data_dir, "reports"), exist_ok=True)
            rf = os.path.join(controller.cost_data_dir, "reports",
                              f"report-{datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S')}.json")
            with open(rf, "w") as f:
                json.dump(asdict(report), f, indent=2, default=str)
            for rec in report.recommendations[:5]:
                logger.info(f"  [{rec['priority'].upper()}] {rec['recommendation_type']}: {rec['resource_name']}")
            time.sleep(controller.report_interval)
        except KeyboardInterrupt:
            break
        except Exception as e:
            logger.error(f"Error: {e}")
            time.sleep(60)

if __name__ == "__main__":
    main()