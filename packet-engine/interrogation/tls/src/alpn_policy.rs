//! ALPN Policy Engine — configurable allow/deny lists per port.
//!
//! Maps JA4+ fingerprints to ALPN policies, enforcing which protocols are
//! permitted on each port. Designed for zero external dependencies; loads
//! configuration from a TOML file (parsed manually).
//!
//! # Policy Model
//!
//! ```text
//! [policies.default]
//! action = "allow"
//! alpn_list = ["h2", "grpc", "http/1.1"]
//!
//! [policies.port_8443]
//! action = "deny"
//! alpn_list = ["grpc"]
//!
//! [policies.ja4_custom]
//! ja4_prefix = "t13d1516h2"
//! action = "allow"
//! alpn_list = ["h2", "grpc"]
//! ```

use std::collections::HashMap;
use std::str::FromStr;

/// Policy action for ALPN matching.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PolicyAction {
    /// Allow the connection if ALPN matches
    Allow,
    /// Deny the connection if ALPN matches
    Deny,
}

impl std::str::FromStr for PolicyAction {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "allow" => Ok(Self::Allow),
            "deny" => Ok(Self::Deny),
            _ => Err(()),
        }
    }
}

/// ALPN policy entry.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AlpnPolicy {
    /// Policy action
    pub action: PolicyAction,
    /// Allowed/denied ALPN protocols
    pub alpn_list: Vec<String>,
    /// Optional JA4 prefix to match (if empty, applies to all)
    pub ja4_prefix: Option<String>,
}

impl AlpnPolicy {
    /// Create a new policy.
    pub fn new(action: PolicyAction, alpn_list: Vec<String>) -> Self {
        Self {
            action,
            alpn_list,
            ja4_prefix: None,
        }
    }

    /// Create a policy with JA4 prefix matching.
    pub fn with_ja4_prefix(action: PolicyAction, alpn_list: Vec<String>, prefix: String) -> Self {
        Self {
            action,
            alpn_list,
            ja4_prefix: Some(prefix),
        }
    }

    /// Check if a given ALPN value is allowed by this policy.
    pub fn check_alpn(&self, alpn: &str) -> bool {
        match self.action {
            PolicyAction::Allow => {
                self.alpn_list.is_empty() || self.alpn_list.iter().any(|a| a == alpn)
            }
            PolicyAction::Deny => {
                !self.alpn_list.is_empty() && !self.alpn_list.iter().any(|a| a == alpn)
            }
        }
    }
}

/// ALPN Policy Engine — manages policies per port.
#[derive(Clone, Debug)]
pub struct AlpnPolicyEngine {
    /// Default policy (applies when no port-specific policy exists)
    pub default_policy: AlpnPolicy,
    /// Port-specific policies
    pub port_policies: HashMap<u16, AlpnPolicy>,
    /// JA4-specific policies (applied after port check)
    pub ja4_policies: Vec<AlpnPolicy>,
}

impl AlpnPolicyEngine {
    /// Create a new engine with a default "allow all" policy.
    pub fn new() -> Self {
        Self {
            default_policy: AlpnPolicy::new(PolicyAction::Allow, vec![]),
            port_policies: HashMap::new(),
            ja4_policies: Vec::new(),
        }
    }

    /// Create an engine with a restrictive default.
    pub fn restrictive() -> Self {
        Self {
            default_policy: AlpnPolicy::new(PolicyAction::Deny, vec![]),
            port_policies: HashMap::new(),
            ja4_policies: Vec::new(),
        }
    }

    /// Set the default policy.
    pub fn set_default(&mut self, policy: AlpnPolicy) {
        self.default_policy = policy;
    }

    /// Set a policy for a specific port.
    pub fn set_port_policy(&mut self, port: u16, policy: AlpnPolicy) {
        self.port_policies.insert(port, policy);
    }

    /// Add a JA4-specific policy.
    pub fn add_ja4_policy(&mut self, policy: AlpnPolicy) {
        self.ja4_policies.push(policy);
    }

    /// Check if an ALPN protocol is allowed on a given port.
    ///
    /// Evaluation order:
    /// 1. JA4-specific policies (if JA4 fingerprint provided)
    /// 2. Port-specific policies
    /// 3. Default policy
    pub fn is_alpn_allowed(&self, port: u16, alpn: &str, ja4: Option<&str>) -> bool {
        // 1. Check JA4-specific policies
        if let Some(ja4_fp) = ja4 {
            for policy in &self.ja4_policies {
                if let Some(ref prefix) = policy.ja4_prefix {
                    if ja4_fp.starts_with(prefix) {
                        return policy.check_alpn(alpn);
                    }
                }
            }
        }

        // 2. Check port-specific policy
        if let Some(policy) = self.port_policies.get(&port) {
            return policy.check_alpn(alpn);
        }

        // 3. Fall back to default
        self.default_policy.check_alpn(alpn)
    }

    /// Parse a minimal TOML policy file.
    ///
    /// Supports:
    /// ```toml
    /// [policies.default]
    /// action = "allow"
    /// alpn_list = ["h2", "grpc"]
    ///
    /// [policies.port_8443]
    /// action = "deny"
    /// alpn_list = ["grpc"]
    ///
    /// [policies.ja4_custom]
    /// ja4_prefix = "t13d1516h2"
    /// action = "allow"
    /// alpn_list = ["h2"]
    /// ```
    pub fn from_toml(toml: &str) -> Result<Self, PolicyParseError> {
        let mut engine = Self::new();
        let mut current_section = String::new();
        let mut current_action = PolicyAction::Allow;
        let mut current_alpn_list = Vec::new();
        let mut current_ja4_prefix: Option<String> = None;

        for line in toml.lines() {
            let line = line.trim();

            // Skip empty lines and comments
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            // Section header
            if line.starts_with('[') && line.ends_with(']') {
                // Flush previous section
                if !current_section.is_empty() {
                    let policy = if let Some(ref prefix) = current_ja4_prefix {
                        AlpnPolicy::with_ja4_prefix(
                            current_action,
                            current_alpn_list.clone(),
                            prefix.clone(),
                        )
                    } else {
                        AlpnPolicy::new(current_action, current_alpn_list.clone())
                    };
                    engine = apply_section(&engine, &current_section, policy);
                }

                current_section = line[1..line.len() - 1].trim().to_string();
                current_action = PolicyAction::Allow;
                current_alpn_list.clear();
                current_ja4_prefix = None;
                continue;
            }

            // Key = value
            if let Some((key, value)) = line.split_once('=') {
                let key = key.trim();
                let value = unquote(value.trim());

                match key {
                    "action" => {
                        current_action = PolicyAction::from_str(&value)
                            .map_err(|_| PolicyParseError::InvalidAction(value.clone()))?;
                    }
                    "alpn_list" => {
                        current_alpn_list = parse_string_array(&value);
                    }
                    "ja4_prefix" => {
                        current_ja4_prefix = Some(value);
                    }
                    _ => {}
                }
            }
        }

        // Flush last section
        if !current_section.is_empty() {
            let policy = if let Some(ref prefix) = current_ja4_prefix {
                AlpnPolicy::with_ja4_prefix(
                    current_action,
                    current_alpn_list.clone(),
                    prefix.clone(),
                )
            } else {
                AlpnPolicy::new(current_action, current_alpn_list.clone())
            };
            engine = apply_section(&engine, &current_section, policy);
        }

        Ok(engine)
    }
}

impl Default for AlpnPolicyEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Apply a parsed section to the engine.
fn apply_section(engine: &AlpnPolicyEngine, section: &str, policy: AlpnPolicy) -> AlpnPolicyEngine {
    let mut engine = engine.clone();
    match section {
        "policies.default" => {
            engine.set_default(policy);
        }
        s if s.starts_with("policies.port_") => {
            if let Ok(port) = s["policies.port_".len()..].parse::<u16>() {
                engine.set_port_policy(port, policy);
            }
        }
        s if s.starts_with("policies.ja4_") => {
            engine.add_ja4_policy(policy);
        }
        _ => {}
    }
    engine
}

/// Parse a TOML-style string array: ["a", "b", "c"]
fn parse_string_array(s: &str) -> Vec<String> {
    let s = s.trim();
    if !s.starts_with('[') || !s.ends_with(']') {
        return Vec::new();
    }
    let inner = &s[1..s.len() - 1];
    inner
        .split(',')
        .map(|item| unquote(item.trim()))
        .filter(|s| !s.is_empty())
        .collect()
}

/// Remove surrounding quotes from a string.
fn unquote(s: &str) -> String {
    if (s.starts_with('"') && s.ends_with('"')) || (s.starts_with('\'') && s.ends_with('\'')) {
        s[1..s.len() - 1].to_string()
    } else {
        s.to_string()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PolicyParseError {
    InvalidAction(String),
    MissingField(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_policy_allow() {
        let policy = AlpnPolicy::new(
            PolicyAction::Allow,
            vec!["h2".to_string(), "grpc".to_string()],
        );
        assert!(policy.check_alpn("h2"));
        assert!(policy.check_alpn("grpc"));
        assert!(!policy.check_alpn("http/1.1"));
    }

    #[test]
    fn test_policy_deny() {
        let policy = AlpnPolicy::new(PolicyAction::Deny, vec!["grpc".to_string()]);
        assert!(!policy.check_alpn("grpc"));
        assert!(policy.check_alpn("h2"));
    }

    #[test]
    fn test_engine_default_only() {
        let engine = AlpnPolicyEngine::new();
        // Empty default = allow all
        assert!(engine.is_alpn_allowed(443, "h2", None));
        assert!(engine.is_alpn_allowed(443, "grpc", None));
    }

    #[test]
    fn test_engine_port_specific() {
        let mut engine = AlpnPolicyEngine::new();
        engine.set_port_policy(
            8443,
            AlpnPolicy::new(PolicyAction::Deny, vec!["grpc".to_string()]),
        );
        assert!(!engine.is_alpn_allowed(8443, "grpc", None));
        assert!(engine.is_alpn_allowed(8443, "h2", None));
        assert!(engine.is_alpn_allowed(443, "grpc", None)); // different port
    }

    #[test]
    fn test_engine_ja4_specific() {
        let mut engine = AlpnPolicyEngine::new();
        engine.add_ja4_policy(AlpnPolicy::with_ja4_prefix(
            PolicyAction::Allow,
            vec!["h2".to_string()],
            "t13d1516h2".to_string(),
        ));
        assert!(engine.is_alpn_allowed(443, "h2", Some("t13d1516h2_abc_def")));
        assert!(!engine.is_alpn_allowed(443, "grpc", Some("t13d1516h2_abc_def")));
        assert!(engine.is_alpn_allowed(443, "h2", Some("t12d1011h2_abc_def"))); // different prefix
    }

    #[test]
    fn test_from_toml_default() {
        let toml = r#"
[policies.default]
action = "allow"
alpn_list = ["h2", "grpc"]
"#;
        let engine = AlpnPolicyEngine::from_toml(toml).unwrap();
        assert!(engine.is_alpn_allowed(443, "h2", None));
        assert!(engine.is_alpn_allowed(443, "grpc", None));
        assert!(!engine.is_alpn_allowed(443, "http/1.1", None));
    }

    #[test]
    fn test_from_toml_port_specific() {
        let toml = r#"
[policies.default]
action = "allow"
alpn_list = ["h2"]

[policies.port_8443]
action = "deny"
alpn_list = ["grpc"]
"#;
        let engine = AlpnPolicyEngine::from_toml(toml).unwrap();
        assert!(engine.is_alpn_allowed(443, "h2", None));
        assert!(!engine.is_alpn_allowed(8443, "grpc", None));
        assert!(engine.is_alpn_allowed(8443, "h2", None));
    }

    #[test]
    fn test_from_toml_ja4() {
        let toml = r#"
[policies.default]
action = "deny"
alpn_list = []

[policies.ja4_known]
ja4_prefix = "t13d1516h2"
action = "allow"
alpn_list = ["h2", "grpc"]
"#;
        let engine = AlpnPolicyEngine::from_toml(toml).unwrap();
        assert!(engine.is_alpn_allowed(443, "h2", Some("t13d1516h2_abc_def")));
        assert!(!engine.is_alpn_allowed(443, "h2", Some("t12d1011h2_abc_def")));
    }

    #[test]
    fn test_parse_string_array() {
        assert_eq!(
            parse_string_array(r#"["h2", "grpc", "http/1.1"]"#),
            vec!["h2".to_string(), "grpc".to_string(), "http/1.1".to_string()]
        );
        assert_eq!(parse_string_array("[]"), Vec::<String>::new());
        assert_eq!(
            parse_string_array(r#"["single"]"#),
            vec!["single".to_string()]
        );
    }

    #[test]
    fn test_unquote() {
        assert_eq!(unquote(r#""hello""#), "hello");
        assert_eq!(unquote("'hello'"), "hello");
        assert_eq!(unquote("hello"), "hello");
    }

    #[test]
    fn test_action_from_str() {
        assert_eq!(PolicyAction::from_str("allow"), Ok(PolicyAction::Allow));
        assert_eq!(PolicyAction::from_str("DENY"), Ok(PolicyAction::Deny));
        assert_eq!(PolicyAction::from_str("invalid"), Err(()));
    }

    #[test]
    fn test_engine_restrictive() {
        let engine = AlpnPolicyEngine::restrictive();
        assert!(!engine.is_alpn_allowed(443, "h2", None));
        assert!(!engine.is_alpn_allowed(443, "grpc", None));
    }
}
