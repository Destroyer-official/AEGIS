#!/usr/bin/env python3
"""AEGIS Cluster Registry Controller - health checks remote clusters"""

import os, sys, json, time, logging, subprocess, tempfile
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

class ClusterRegistryController:
    def __init__(self):
        try:
            config.load_incluster_config()
        except config.ConfigException:
            config.load_kube_config()
        self.core_v1 = client.CoreV1Api()
        self.custom_api = client.CustomObjectsApi()
        self.group = "aegis.io"
        self.version = "v1alpha1"
        self.plural = "clusterregistries"

    def list_registries(self) -> List[Dict]:
        try:
            result = self.custom_api.list_cluster_custom_object(
                group=self.group, version=self.version, plural=self.plural)
            return result.get("items", [])
        except ApiException as e:
            logger.error(f"Failed to list ClusterRegistries: {e}")
            return []

    def update_status(self, name: str, status: Dict):
        try:
            self.custom_api.patch_cluster_custom_object_status(
                group=self.group, version=self.version, plural=self.plural,
                name=name, body={"status": status})
        except ApiException as e:
            logger.error(f"Failed to update status for {name}: {e}")

    def health_check_cluster(self, endpoint: str, kubeconfig_secret: Optional[str] = None) -> Dict:
        result = {"ready": False, "latencyMs": 0, "lastHealthCheck": datetime.now(timezone.utc).isoformat()}
        try:
            kubeconfig_data = None
            if kubeconfig_secret:
                try:
                    secret = self.core_v1.read_namespaced_secret(kubeconfig_secret, "default")
                    kubeconfig_data = secret.data.get("kubeconfig")
                    if kubeconfig_data:
                        import base64
                        kubeconfig_data = base64.b64decode(kubeconfig_data).decode("utf-8")
                except ApiException:
                    logger.warning(f"Secret {kubeconfig_secret} not found, using in-cluster config")

            start = time.time()
            if kubeconfig_data:
                with tempfile.NamedTemporaryFile(mode='w', suffix='.kubeconfig', delete=False) as f:
                    f.write(kubeconfig_data)
                    kubeconfig_path = f.name
                try:
                    cmd = ["kubectl", "--kubeconfig", kubeconfig_path, "get", "pods", "-A", "--no-headers", "--request-timeout=5s"]
                    proc = subprocess.run(cmd, capture_output=True, text=True, timeout=10)
                finally:
                    os.unlink(kubeconfig_path)
            else:
                cmd = ["kubectl", "get", "pods", "-A", "--no-headers", "--request-timeout=5s"]
                proc = subprocess.run(cmd, capture_output=True, text=True, timeout=10)

            latency = int((time.time() - start) * 1000)
            result["ready"] = proc.returncode == 0
            result["latencyMs"] = latency
        except (subprocess.TimeoutExpired, Exception) as e:
            logger.warning(f"Health check failed for {endpoint}: {e}")
        return result

    def reconcile(self, registry: Dict):
        name = registry["metadata"]["name"]
        spec = registry.get("spec", {})
        clusters = spec.get("clusters", [])
        logger.info(f"Reconciling: {name} ({len(clusters)} clusters)")
        cluster_status = []
        ready_count = 0
        for cluster in clusters:
            health = self.health_check_cluster(cluster["endpoint"], cluster.get("kubeconfigSecret"))
            health["name"] = cluster["name"]
            cluster_status.append(health)
            if health["ready"]:
                ready_count += 1
        status = {
            "phase": "Active" if ready_count > 0 else "Degraded",
            "readyClusters": ready_count,
            "lastSyncTime": datetime.now(timezone.utc).isoformat(),
            "clusterStatus": cluster_status,
            "conditions": [{"type": "Ready",
                          "status": "True" if ready_count > 0 else "False",
                          "lastTransitionTime": datetime.now(timezone.utc).isoformat(),
                          "reason": "ClustersReady" if ready_count > 0 else "NoReadyClusters",
                          "message": f"{ready_count}/{len(clusters)} clusters ready"}]}
        self.update_status(name, status)
        logger.info(f"Updated: {ready_count}/{len(clusters)} ready")

    def run(self):
        logger.info("Starting Cluster Registry Controller")
        while True:
            try:
                for registry in self.list_registries():
                    self.reconcile(registry)
            except Exception as e:
                logger.error(f"Error: {e}")
            time.sleep(30)

if __name__ == "__main__":
    ClusterRegistryController().run()