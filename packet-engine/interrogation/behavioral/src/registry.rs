//! MLflow-compatible model registry with canary routing and auto-rollback.
//!
//! Provides a local model registry that tracks model versions, supports
//! A/B testing via canary routing (10% traffic to v2), and automatically
//! rolls back on false-positive (FP) spike detection.

use std::collections::HashMap;
use std::fmt;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use crate::llm::ModelFamily;

// ─── Errors ──────────────────────────────────────────────────────────────────

/// Model registry errors.
#[derive(Debug, Clone, thiserror::Error)]
pub enum RegistryError {
    #[error("model not found: {0}")]
    ModelNotFound(String),

    #[error("version not found: {0}")]
    VersionNotFound(String),

    #[error("invalid canary weight: {0} (must be 0.0-1.0)")]
    InvalidCanaryWeight(f64),

    #[error("rollback threshold exceeded: FP rate {current:.2} > {threshold:.2}")]
    RollbackTriggered { current: f64, threshold: f64 },
}

impl PartialEq for RegistryError {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::ModelNotFound(a), Self::ModelNotFound(b)) => a == b,
            (Self::VersionNotFound(a), Self::VersionNotFound(b)) => a == b,
            (Self::InvalidCanaryWeight(a), Self::InvalidCanaryWeight(b)) => {
                a.to_bits() == b.to_bits()
            }
            (
                Self::RollbackTriggered {
                    current: c1,
                    threshold: t1,
                },
                Self::RollbackTriggered {
                    current: c2,
                    threshold: t2,
                },
            ) => c1.to_bits() == c2.to_bits() && t1.to_bits() == t2.to_bits(),
            _ => false,
        }
    }
}

impl Eq for RegistryError {}

// ─── Model Version ───────────────────────────────────────────────────────────

/// Status of a model version in the registry.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ModelStatus {
    /// Model is being validated
    Staging,
    /// Model is active and serving traffic
    Active,
    /// Model is in canary (receiving partial traffic)
    Canary,
    /// Model has been archived
    Archived,
    /// Model was rolled back due to issues
    RolledBack,
}

impl fmt::Display for ModelStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Staging => write!(f, "staging"),
            Self::Active => write!(f, "active"),
            Self::Canary => write!(f, "canary"),
            Self::Archived => write!(f, "archived"),
            Self::RolledBack => write!(f, "rolled_back"),
        }
    }
}

/// A registered model version.
#[derive(Clone, Debug)]
pub struct ModelVersion {
    /// Model name (e.g., "refusal-vector-classifier")
    pub name: String,
    /// Version string (e.g., "1.0.0", "v2-beta")
    pub version: String,
    /// Model family this version classifies
    pub family: ModelFamily,
    /// Current status
    pub status: ModelStatus,
    /// Creation timestamp (seconds since UNIX epoch)
    pub created_at: u64,
    /// Last updated timestamp
    pub updated_at: u64,
    /// MLflow-compatible run ID (for tracking)
    pub run_id: Option<String>,
    /// Model artifacts path (local filesystem)
    pub artifact_path: Option<String>,
    /// Metrics recorded at registration
    pub metrics: HashMap<String, f64>,
    /// Tags (key-value metadata)
    pub tags: HashMap<String, String>,
}

impl ModelVersion {
    /// Create a new model version.
    pub fn new(name: &str, version: &str, family: ModelFamily) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Self {
            name: name.to_string(),
            version: version.to_string(),
            family,
            status: ModelStatus::Staging,
            created_at: now,
            updated_at: now,
            run_id: None,
            artifact_path: None,
            metrics: HashMap::new(),
            tags: HashMap::new(),
        }
    }

    /// Record a metric for this version.
    pub fn record_metric(&mut self, key: &str, value: f64) {
        self.metrics.insert(key.to_string(), value);
        self.updated_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
    }

    /// Get a metric value.
    pub fn get_metric(&self, key: &str) -> Option<f64> {
        self.metrics.get(key).copied()
    }

    /// Set a tag.
    pub fn set_tag(&mut self, key: &str, value: &str) {
        self.tags.insert(key.to_string(), value.to_string());
    }
}

// ─── Canary Config ───────────────────────────────────────────────────────────

/// Canary routing configuration for A/B testing.
#[derive(Clone, Debug)]
pub struct CanaryConfig {
    /// Weight for canary version (0.0 = no traffic, 1.0 = all traffic)
    pub canary_weight: f64,
    /// FP rate threshold that triggers rollback
    pub rollback_threshold: f64,
    /// Minimum number of observations before rollback can trigger
    pub min_observations: usize,
    /// Evaluation window duration
    pub evaluation_window: Duration,
}

impl Default for CanaryConfig {
    fn default() -> Self {
        Self {
            canary_weight: 0.1,       // 10% traffic to canary
            rollback_threshold: 0.05, // 5% FP rate triggers rollback
            min_observations: 100,
            evaluation_window: Duration::from_secs(3600), // 1 hour
        }
    }
}

/// Routing decision for a single request.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RoutingDecision {
    /// Route to primary (v1)
    Primary,
    /// Route to canary (v2)
    Canary,
}

impl fmt::Display for RoutingDecision {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Primary => write!(f, "primary"),
            Self::Canary => write!(f, "canary"),
        }
    }
}

// ─── Observation ─────────────────────────────────────────────────────────────

/// A single observation (prediction result) for tracking FP rates.
#[derive(Clone, Debug)]
pub struct Observation {
    /// Whether this was a true positive (correctly identified threat)
    pub true_positive: bool,
    /// Whether this was a false positive (incorrectly flagged benign)
    pub false_positive: bool,
    /// Timestamp
    pub timestamp: Instant,
    /// Confidence score
    pub confidence: f64,
}

// ─── Model Registry ──────────────────────────────────────────────────────────

/// MLflow-compatible model registry with canary routing.
pub struct ModelRegistry {
    /// Registered models: name -> versions
    models: HashMap<String, Vec<ModelVersion>>,
    /// Canary config per model
    canary_configs: HashMap<String, CanaryConfig>,
    /// Observations per model version (for FP tracking)
    observations: HashMap<String, Vec<Observation>>,
    /// Currently active canary version per model
    active_canary: HashMap<String, String>,
}

impl ModelRegistry {
    /// Create a new empty registry.
    pub fn new() -> Self {
        Self {
            models: HashMap::new(),
            canary_configs: HashMap::new(),
            observations: HashMap::new(),
            active_canary: HashMap::new(),
        }
    }

    /// Register a new model version.
    pub fn register_model(&mut self, version: ModelVersion) -> Result<(), RegistryError> {
        let name = version.name.clone();
        let ver = version.version.clone();

        // Check for duplicate version
        if let Some(versions) = self.models.get(&name) {
            if versions.iter().any(|v| v.version == ver) {
                return Err(RegistryError::VersionNotFound(format!(
                    "{}.{} already exists",
                    name, ver
                )));
            }
        }

        self.models.entry(name.clone()).or_default().push(version);
        self.observations
            .entry(format!("{}.{}", name, ver))
            .or_default();

        Ok(())
    }

    /// Get a model version by name and version string.
    pub fn get_version(&self, name: &str, version: &str) -> Result<&ModelVersion, RegistryError> {
        self.models
            .get(name)
            .and_then(|versions| versions.iter().find(|v| v.version == version))
            .ok_or_else(|| RegistryError::VersionNotFound(format!("{}.{}", name, version)))
    }

    /// Get a mutable reference to a model version.
    pub fn get_version_mut(
        &mut self,
        name: &str,
        version: &str,
    ) -> Result<&mut ModelVersion, RegistryError> {
        self.models
            .get_mut(name)
            .and_then(|versions| versions.iter_mut().find(|v| v.version == version))
            .ok_or_else(|| RegistryError::VersionNotFound(format!("{}.{}", name, version)))
    }

    /// List all versions for a model.
    pub fn list_versions(&self, name: &str) -> Vec<&ModelVersion> {
        self.models
            .get(name)
            .map(|versions| versions.iter().collect())
            .unwrap_or_default()
    }

    /// Get the active (non-archived, non-rolled-back) versions for a model.
    pub fn get_active_versions(&self, name: &str) -> Vec<&ModelVersion> {
        self.models
            .get(name)
            .map(|versions| {
                versions
                    .iter()
                    .filter(|v| v.status == ModelStatus::Active || v.status == ModelStatus::Canary)
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Transition a model version to a new status.
    pub fn set_status(
        &mut self,
        name: &str,
        version: &str,
        status: ModelStatus,
    ) -> Result<(), RegistryError> {
        let model = self.get_version_mut(name, version)?;
        model.status = status;
        model.updated_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        Ok(())
    }

    /// Set up canary routing for a model.
    pub fn configure_canary(
        &mut self,
        name: &str,
        canary_version: &str,
        config: CanaryConfig,
    ) -> Result<(), RegistryError> {
        if config.canary_weight < 0.0 || config.canary_weight > 1.0 {
            return Err(RegistryError::InvalidCanaryWeight(config.canary_weight));
        }

        // Verify canary version exists and is active
        let model = self.get_version(name, canary_version)?;
        if model.status != ModelStatus::Active && model.status != ModelStatus::Staging {
            return Err(RegistryError::VersionNotFound(format!(
                "{}.{} must be active or staging for canary",
                name, canary_version
            )));
        }

        self.canary_configs.insert(name.to_string(), config);
        self.active_canary
            .insert(name.to_string(), canary_version.to_string());

        // Set canary version status
        self.set_status(name, canary_version, ModelStatus::Canary)?;

        Ok(())
    }

    /// Make a routing decision for a request (deterministic based on request hash).
    pub fn route(&self, name: &str, request_hash: u64) -> Result<RoutingDecision, RegistryError> {
        let config = self
            .canary_configs
            .get(name)
            .ok_or_else(|| RegistryError::ModelNotFound(name.to_string()))?;

        // Use request hash to determine routing (deterministic)
        let bucket = (request_hash % 1000) as f64 / 1000.0;

        if bucket < config.canary_weight {
            Ok(RoutingDecision::Canary)
        } else {
            Ok(RoutingDecision::Primary)
        }
    }

    /// Record an observation for a model version.
    pub fn record_observation(
        &mut self,
        name: &str,
        version: &str,
        observation: Observation,
    ) -> Result<(), RegistryError> {
        let key = format!("{}.{}", name, version);
        self.observations.entry(key).or_default().push(observation);
        Ok(())
    }

    /// Compute FP rate for a model version within the evaluation window.
    pub fn compute_fp_rate(&self, name: &str, version: &str) -> Result<f64, RegistryError> {
        let config = self
            .canary_configs
            .get(name)
            .ok_or_else(|| RegistryError::ModelNotFound(name.to_string()))?;

        let key = format!("{}.{}", name, version);
        let observations = self
            .observations
            .get(&key)
            .ok_or(RegistryError::VersionNotFound(key))?;

        let cutoff = Instant::now() - config.evaluation_window;
        let recent: Vec<&Observation> = observations
            .iter()
            .filter(|o| o.timestamp >= cutoff)
            .collect();

        if recent.len() < config.min_observations {
            return Ok(0.0); // Not enough data
        }

        let fp_count = recent.iter().filter(|o| o.false_positive).count();
        Ok(fp_count as f64 / recent.len() as f64)
    }

    /// Check if rollback should trigger for the canary version.
    pub fn check_rollback(&self, name: &str) -> Result<Option<String>, RegistryError> {
        let canary_version = self
            .active_canary
            .get(name)
            .ok_or_else(|| RegistryError::ModelNotFound(name.to_string()))?;

        let config = self
            .canary_configs
            .get(name)
            .ok_or_else(|| RegistryError::ModelNotFound(name.to_string()))?;

        let fp_rate = self.compute_fp_rate(name, canary_version)?;

        let key = format!("{}.{}", name, canary_version);
        let observations = self.observations.get(&key).map(|v| v.len()).unwrap_or(0);

        if observations >= config.min_observations && fp_rate > config.rollback_threshold {
            Ok(Some(canary_version.clone()))
        } else {
            Ok(None)
        }
    }

    /// Execute rollback: archive canary, promote previous active version.
    pub fn execute_rollback(&mut self, name: &str) -> Result<String, RegistryError> {
        let canary_version = self
            .active_canary
            .get(name)
            .ok_or_else(|| RegistryError::ModelNotFound(name.to_string()))?
            .clone();

        // Archive the canary
        self.set_status(name, &canary_version, ModelStatus::RolledBack)?;

        // Find the previous active version and promote it
        if let Some(versions) = self.models.get(name) {
            if let Some(prev) = versions
                .iter()
                .rev()
                .find(|v| v.status == ModelStatus::Active)
            {
                let prev_version = prev.version.clone();
                self.active_canary.remove(name);
                return Ok(format!("Rolled back {} to {}", name, prev_version));
            }
        }

        self.active_canary.remove(name);
        Ok(format!("Rolled back {} (no previous active version)", name))
    }

    /// Get registry summary.
    pub fn summary(&self) -> RegistrySummary {
        let mut total_models = 0;
        let mut total_versions = 0;
        let mut status_counts = HashMap::new();

        for versions in self.models.values() {
            total_models += 1;
            for v in versions {
                total_versions += 1;
                *status_counts.entry(v.status.to_string()).or_insert(0) += 1;
            }
        }

        RegistrySummary {
            total_models,
            total_versions,
            status_counts,
            active_canaries: self.active_canary.len(),
        }
    }
}

impl Default for ModelRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Summary of the model registry state.
#[derive(Clone, Debug)]
pub struct RegistrySummary {
    pub total_models: usize,
    pub total_versions: usize,
    pub status_counts: HashMap<String, usize>,
    pub active_canaries: usize,
}

impl fmt::Display for RegistrySummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Registry: {} models, {} versions, {} canaries active",
            self.total_models, self.total_versions, self.active_canaries
        )
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn test_version(name: &str, version: &str) -> ModelVersion {
        ModelVersion::new(name, version, ModelFamily::Gpt4)
    }

    #[test]
    fn test_register_and_get() {
        let mut registry = ModelRegistry::new();
        let v = test_version("test-model", "1.0.0");
        registry.register_model(v).unwrap();

        let fetched = registry.get_version("test-model", "1.0.0").unwrap();
        assert_eq!(fetched.name, "test-model");
        assert_eq!(fetched.version, "1.0.0");
        assert_eq!(fetched.status, ModelStatus::Staging);
    }

    #[test]
    fn test_duplicate_version_rejected() {
        let mut registry = ModelRegistry::new();
        registry.register_model(test_version("m", "1.0")).unwrap();
        let result = registry.register_model(test_version("m", "1.0"));
        assert!(result.is_err());
    }

    #[test]
    fn test_list_versions() {
        let mut registry = ModelRegistry::new();
        registry.register_model(test_version("m", "1.0")).unwrap();
        registry.register_model(test_version("m", "2.0")).unwrap();

        let versions = registry.list_versions("m");
        assert_eq!(versions.len(), 2);
    }

    #[test]
    fn test_set_status() {
        let mut registry = ModelRegistry::new();
        registry.register_model(test_version("m", "1.0")).unwrap();
        registry
            .set_status("m", "1.0", ModelStatus::Active)
            .unwrap();

        let v = registry.get_version("m", "1.0").unwrap();
        assert_eq!(v.status, ModelStatus::Active);
    }

    #[test]
    fn test_configure_canary() {
        let mut registry = ModelRegistry::new();
        registry.register_model(test_version("m", "1.0")).unwrap();
        registry.register_model(test_version("m", "2.0")).unwrap();
        registry
            .set_status("m", "1.0", ModelStatus::Active)
            .unwrap();

        let config = CanaryConfig::default();
        registry.configure_canary("m", "2.0", config).unwrap();

        let v = registry.get_version("m", "2.0").unwrap();
        assert_eq!(v.status, ModelStatus::Canary);
    }

    #[test]
    fn test_canary_invalid_weight() {
        let mut registry = ModelRegistry::new();
        registry.register_model(test_version("m", "1.0")).unwrap();
        registry
            .set_status("m", "1.0", ModelStatus::Active)
            .unwrap();

        let config = CanaryConfig {
            canary_weight: 1.5,
            ..Default::default()
        };
        let result = registry.configure_canary("m", "1.0", config);
        assert!(result.is_err());
    }

    #[test]
    fn test_routing_decision() {
        let mut registry = ModelRegistry::new();
        registry.register_model(test_version("m", "1.0")).unwrap();
        registry.register_model(test_version("m", "2.0")).unwrap();
        registry
            .set_status("m", "1.0", ModelStatus::Active)
            .unwrap();

        let config = CanaryConfig {
            canary_weight: 0.1,
            ..Default::default()
        };
        registry.configure_canary("m", "2.0", config).unwrap();

        // Hash 0-99 should route to canary (10% of 1000)
        let canary_count = (0..1000)
            .filter(|&h| registry.route("m", h).unwrap() == RoutingDecision::Canary)
            .count();
        // Should be approximately 100 (10%)
        assert!((90..=110).contains(&canary_count));
    }

    #[test]
    fn test_record_observation() {
        let mut registry = ModelRegistry::new();
        registry.register_model(test_version("m", "1.0")).unwrap();

        // Need a canary config for compute_fp_rate to work
        let config = CanaryConfig {
            min_observations: 10,
            ..Default::default()
        };
        // We need a second version for canary, but we can test observations directly
        registry.register_model(test_version("m", "2.0")).unwrap();
        registry
            .set_status("m", "1.0", ModelStatus::Active)
            .unwrap();
        registry.configure_canary("m", "2.0", config).unwrap();

        for _ in 0..50 {
            registry
                .record_observation(
                    "m",
                    "1.0",
                    Observation {
                        true_positive: true,
                        false_positive: false,
                        timestamp: Instant::now(),
                        confidence: 0.9,
                    },
                )
                .unwrap();
        }

        for _ in 0..5 {
            registry
                .record_observation(
                    "m",
                    "1.0",
                    Observation {
                        true_positive: false,
                        false_positive: true,
                        timestamp: Instant::now(),
                        confidence: 0.6,
                    },
                )
                .unwrap();
        }

        let fp_rate = registry.compute_fp_rate("m", "1.0").unwrap();
        // 5 FP out of 55 total = ~9%
        assert!((fp_rate - 0.09).abs() < 0.02);
    }

    #[test]
    fn test_rollback_check() {
        let mut registry = ModelRegistry::new();
        registry.register_model(test_version("m", "1.0")).unwrap();
        registry.register_model(test_version("m", "2.0")).unwrap();
        registry
            .set_status("m", "1.0", ModelStatus::Active)
            .unwrap();

        let config = CanaryConfig {
            canary_weight: 0.1,
            rollback_threshold: 0.05,
            min_observations: 10,
            evaluation_window: Duration::from_secs(3600),
        };
        registry.configure_canary("m", "2.0", config).unwrap();

        // Record 20 observations, 3 false positives (15% FP rate)
        for _ in 0..17 {
            registry
                .record_observation(
                    "m",
                    "2.0",
                    Observation {
                        true_positive: true,
                        false_positive: false,
                        timestamp: Instant::now(),
                        confidence: 0.9,
                    },
                )
                .unwrap();
        }
        for _ in 0..3 {
            registry
                .record_observation(
                    "m",
                    "2.0",
                    Observation {
                        true_positive: false,
                        false_positive: true,
                        timestamp: Instant::now(),
                        confidence: 0.5,
                    },
                )
                .unwrap();
        }

        let should_rollback = registry.check_rollback("m").unwrap();
        assert!(should_rollback.is_some());
        assert_eq!(should_rollback.unwrap(), "2.0");
    }

    #[test]
    fn test_execute_rollback() {
        let mut registry = ModelRegistry::new();
        registry.register_model(test_version("m", "1.0")).unwrap();
        registry.register_model(test_version("m", "2.0")).unwrap();
        registry
            .set_status("m", "1.0", ModelStatus::Active)
            .unwrap();

        let config = CanaryConfig::default();
        registry.configure_canary("m", "2.0", config).unwrap();

        let result = registry.execute_rollback("m").unwrap();
        assert!(result.contains("1.0"));

        let v = registry.get_version("m", "2.0").unwrap();
        assert_eq!(v.status, ModelStatus::RolledBack);
    }

    #[test]
    fn test_summary() {
        let mut registry = ModelRegistry::new();
        registry.register_model(test_version("m1", "1.0")).unwrap();
        registry.register_model(test_version("m1", "2.0")).unwrap();
        registry.register_model(test_version("m2", "1.0")).unwrap();

        let summary = registry.summary();
        assert_eq!(summary.total_models, 2);
        assert_eq!(summary.total_versions, 3);
    }

    #[test]
    fn test_model_not_found() {
        let registry = ModelRegistry::new();
        let result = registry.get_version("nonexistent", "1.0");
        assert!(result.is_err());
    }

    #[test]
    fn test_version_not_found() {
        let mut registry = ModelRegistry::new();
        registry.register_model(test_version("m", "1.0")).unwrap();
        let result = registry.get_version("m", "9.0");
        assert!(result.is_err());
    }

    #[test]
    fn test_get_active_versions() {
        let mut registry = ModelRegistry::new();
        registry.register_model(test_version("m", "1.0")).unwrap();
        registry.register_model(test_version("m", "2.0")).unwrap();
        registry.register_model(test_version("m", "3.0")).unwrap();

        registry
            .set_status("m", "1.0", ModelStatus::Active)
            .unwrap();
        registry
            .set_status("m", "2.0", ModelStatus::Active)
            .unwrap();
        // 3.0 stays Staging

        let active = registry.get_active_versions("m");
        assert_eq!(active.len(), 2);
    }
}
