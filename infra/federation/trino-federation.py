#!/usr/bin/env python3
"""AEGIS Cross-Cluster Trino Federation - syncs catalog config"""

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

class TrinoFederation:
    def __init__(self):
        try:
            config.load_incluster_config()
        except config.ConfigException:
            config.load_kube_config()
        self.custom_api = client.CustomObjectsApi()
        self.core_v1 = client.CoreV1Api()
        self.group = "aegis.io"
        self.version = "v1alpha1"
        self.plural = "clusterregistries"

    def get_cluster_registry(self) -> Optional[Dict]:
        try:
            result = self.custom_api.list_cluster_custom_object(
                group=self.group, version=self.version, plural=self.plural)
            items = result.get("items", [])
            return items[0] if items else None
        except ApiException as e:
            logger.error(f"Failed to get ClusterRegistry: {e}")
            return None

    def generate_trino_catalog_config(self, registry: Dict) -> str:
        spec = registry.get("spec", {})
        trino_spec = spec.get("trinoFederation", {})
        clusters = spec.get("clusters", [])
        catalog_name = trino_spec.get("catalogName", "aegis-federated")
        lines = [f"# AEGIS Federated Trino Catalog", f"# Generated: {datetime.now(timezone.utc).isoformat()}", ""]
        for cluster in clusters:
            prefix = cluster["name"].replace("-", "_")
            endpoint = cluster["endpoint"]
            region = cluster.get("region", "unknown")
            lines.append(f"# Cluster: {cluster['name']} ({region})")
            lines.append(f"iceberg.nessie-catalog.uri.{prefix}=http://{cluster['name']}-aegis-polaris:19120/api/v1")
            lines.append(f"iceberg.nessie-catalog.ref.{prefix}=main")
            lines.append(f"iceberg.nessie-catalog.default-warehouse-dir.{prefix}=s3://aegis-warehouse/")
            lines.append("")
        return "\n".join(lines)

    def generate_trino_catalog_sql(self, registry: Dict) -> str:
        spec = registry.get("spec", {})
        trino_spec = spec.get("trinoFederation", {})
        catalog_name = trino_spec.get("catalogName", "aegis-federated")
        clusters = spec.get("clusters", [])
        lines = [f"-- AEGIS Cross-Cluster Trino Federation",
                 f"-- Generated: {datetime.now(timezone.utc).isoformat()}", ""]
        for cluster in clusters:
            cat = f"aegis_{cluster['name'].replace('-', '_')}"
            lines.append(f"-- Cluster: {cluster['name']} ({cluster.get('region', 'unknown')})")
            lines.append(f"CREATE SCHEMA IF NOT EXISTS {cat}.aegis;")
            lines.append("")
        lines.append(f"-- Federated view across all clusters")
        parts = []
        for cluster in clusters:
            cat = f"aegis_{cluster['name'].replace('-', '_')}"
            parts.append(f"SELECT '{cluster['name']}' as cluster_name, * FROM {cat}.aegis.hosts")
        lines.append(f"CREATE OR REPLACE VIEW {catalog_name}.aegis.federated_hosts AS")
        lines.append(" UNION ALL ".join(parts) + ";")
        return "\n".join(lines)

    def sync_catalogs(self, registry: Dict):
        spec = registry.get("spec", {})
        trino_spec = spec.get("trinoFederation", {})
        if not trino_spec.get("enabled", True):
            return
        clusters = spec.get("clusters", [])
        logger.info(f"Syncing Trino catalogs for {len(clusters)} clusters")
        config_data = {
            "federation.properties": self.generate_trino_catalog_config(registry),
            "federation.sql": self.generate_trino_catalog_sql(registry)}
        cm_name = "trino-federation-config"
        try:
            try:
                self.core_v1.read_namespaced_config_map(cm_name, "default")
                self.core_v1.patch_namespaced_config_map(cm_name, "default", {"data": config_data})
            except ApiException:
                self.core_v1.create_namespaced_config_map("default",
                    client.V1ConfigMap(metadata=client.V1ObjectMeta(name=cm_name), data=config_data))
            logger.info("Trino federation config synced")
        except ApiException as e:
            logger.error(f"Failed to sync: {e}")

    def run(self):
        logger.info("Starting Trino Federation Sync")
        while True:
            try:
                registry = self.get_cluster_registry()
                if registry:
                    self.sync_catalogs(registry)
            except Exception as e:
                logger.error(f"Error: {e}")
            time.sleep(300)

if __name__ == "__main__":
    TrinoFederation().run()