//! JA4SSH — SSH session type classification from traffic patterns (§6.1).
//!
//! JA4SSH format: `{session_type}_{kex_hash}_{key_type_hash}`
//!
//! Classifies SSH sessions based on:
//! - Key exchange algorithm preferences
//! - Public key types
//! - Structural patterns in authentication failure responses
//! - Response framing patterns

use sha2::{Digest, Sha256};

/// JA4SSH session type.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SshSessionType {
    Interactive,
    Sftp,
    Scp,
    Tunnel,
    PortForward,
    Subsystem,
    Unknown,
}

impl SshSessionType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Interactive => "it",
            Self::Sftp => "sf",
            Self::Scp => "sc",
            Self::Tunnel => "tn",
            Self::PortForward => "pf",
            Self::Subsystem => "ss",
            Self::Unknown => "un",
        }
    }
}

/// JA4SSH fingerprint.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Ja4sshFingerprint {
    pub full: String,
    pub session_type: SshSessionType,
    pub kex_hash: String,
    pub key_type_hash: String,
}

/// SSH session data for JA4SSH computation.
#[derive(Clone, Debug)]
pub struct SshSessionData<'a> {
    /// Key exchange algorithm names in client preference order
    pub kex_algorithms: Vec<&'a str>,
    /// Host key algorithm names
    pub host_key_algorithms: Vec<&'a str>,
    /// Client->Server cipher names
    pub client_ciphers: Vec<&'a str>,
    /// Server->Client cipher names
    pub server_ciphers: Vec<&'a str>,
    /// Client->Server MAC names
    pub client_macs: Vec<&'a str>,
    /// Server->Client MAC names
    pub server_macs: Vec<&'a str>,
    /// Compression methods
    pub compression: Vec<&'a str>,
}

impl Ja4sshFingerprint {
    /// Compute JA4SSH fingerprint from SSH session data.
    pub fn compute(session: &SshSessionData<'_>, session_type: SshSessionType) -> Self {
        // KEX algorithm hash
        let kex_str = session.kex_algorithms.join(",");
        let kex_hash = if kex_str.is_empty() {
            "000000000000".to_string()
        } else {
            let hash = Sha256::digest(kex_str.as_bytes());
            hex::encode(&hash[..6])
        };

        // Host key type hash
        let key_str = session.host_key_algorithms.join(",");
        let key_type_hash = if key_str.is_empty() {
            "000000000000".to_string()
        } else {
            let hash = Sha256::digest(key_str.as_bytes());
            hex::encode(&hash[..6])
        };

        let full = format!("{}_{}_{}", session_type.as_str(), kex_hash, key_type_hash);

        Ja4sshFingerprint {
            full,
            session_type,
            kex_hash,
            key_type_hash,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ja4ssh_format() {
        let session = SshSessionData {
            kex_algorithms: vec![
                "curve25519-sha256",
                "ecdh-sha2-nistp256",
                "diffie-hellman-group14-sha256",
            ],
            host_key_algorithms: vec!["ssh-ed25519", "rsa-sha2-512", "rsa-sha2-256"],
            client_ciphers: vec!["chacha20-poly1305@openssh.com"],
            server_ciphers: vec!["chacha20-poly1305@openssh.com"],
            client_macs: vec!["umac-64@openssh.com"],
            server_macs: vec!["umac-64@openssh.com"],
            compression: vec!["none"],
        };

        let ja4ssh = Ja4sshFingerprint::compute(&session, SshSessionType::Interactive);
        assert!(
            ja4ssh.full.starts_with("it_"),
            "JA4SSH should start with it_, got: {}",
            ja4ssh.full
        );
    }
}
