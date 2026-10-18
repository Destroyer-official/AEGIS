use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Semantic version for signature corpora (MAJOR.MINOR.PATCH).
///
/// **Stability**: Stable since v1.2.0.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct SemVer {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl SemVer {
    pub fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }

    pub fn parse(s: &str) -> Result<Self, StoreError> {
        let parts: Vec<&str> = s.trim_start_matches('v').split('.').collect();
        if parts.len() != 3 {
            return Err(StoreError::InvalidVersion(s.to_string()));
        }
        let major = parts[0]
            .parse()
            .map_err(|_| StoreError::InvalidVersion(s.to_string()))?;
        let minor = parts[1]
            .parse()
            .map_err(|_| StoreError::InvalidVersion(s.to_string()))?;
        let patch = parts[2]
            .parse()
            .map_err(|_| StoreError::InvalidVersion(s.to_string()))?;
        Ok(Self {
            major,
            minor,
            patch,
        })
    }
}

impl std::fmt::Display for SemVer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "v{}.{}.{}", self.major, self.minor, self.patch)
    }
}

/// Version metadata stored as JSON alongside the CSV in git.
///
/// **Stability**: Stable since v1.2.0.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignatureVersion {
    pub version: SemVer,
    pub timestamp: String,
    pub author: String,
    pub description: String,
    pub commit_hash: String,
    pub signature_count: usize,
}

/// Errors from signature store operations.
///
/// **Stability**: Stable since v1.2.0.
#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error("git error: {0}")]
    GitError(String),
    #[error("io error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("invalid version string: {0}")]
    InvalidVersion(String),
    #[error("serde error: {0}")]
    SerdeError(String),
    #[error("store not initialized — run `lzr signatures init`")]
    NotInitialized,
    #[error("version not found: {0}")]
    VersionNotFound(String),
}

/// Git-backed signature store with semantic versioning.
///
/// Stores signatures as CSV + version metadata JSON in a local git repo.
/// Each push creates a tagged commit; rollback reverts to a previous tag.
///
/// # Example
///
/// ```no_run
/// use aegis_lzr::signature_store::{SignatureStore, SemVer};
///
/// let store = SignatureStore::new("/tmp/lzr-sigs");
/// store.init().unwrap();
/// let csv = std::fs::read_to_string("data/signatures.csv").unwrap();
/// let ver = store.push(&csv, SemVer::new(1, 0, 0), "Initial corpus", "dev3").unwrap();
/// ```
///
/// **Stability**: Stable since v1.2.0.
pub struct SignatureStore {
    repo_path: PathBuf,
}

impl SignatureStore {
    /// Create a new store handle at the given directory path.
    ///
    /// **Stability**: Stable since v1.2.0.
    pub fn new(repo_path: impl AsRef<Path>) -> Self {
        Self {
            repo_path: repo_path.as_ref().to_path_buf(),
        }
    }

    /// Initialize the git repository if not already initialized.
    ///
    /// Creates the directory, runs `git init`, and writes an initial commit.
    ///
    /// **Stability**: Stable since v1.2.0.
    pub fn init(&self) -> Result<(), StoreError> {
        fs::create_dir_all(&self.repo_path)?;

        if !self.repo_path.join(".git").exists() {
            self.git(&["init"])?;
            self.git(&["config", "user.email", "lzr@aegis.local"])?;
            self.git(&["config", "user.name", "LZR Signatures"])?;
        }

        let readme = self.repo_path.join("README.md");
        if !readme.exists() {
            fs::write(
                &readme,
                "# LZR Signature Store\n\nGit-backed versioned signature corpus.\n",
            )?;
            self.git(&["add", "."])?;
            self.git(&["commit", "-m", "init: empty signature store"])?;
        }

        Ok(())
    }

    /// Push new signatures to the store.
    ///
    /// Writes the CSV data + version metadata, commits with a semver tag.
    ///
    /// **Stability**: Stable since v1.2.0.
    pub fn push(
        &self,
        csv_data: &str,
        version: SemVer,
        description: &str,
        author: &str,
    ) -> Result<SignatureVersion, StoreError> {
        self.ensure_initialized()?;

        let sig_count = csv_data.lines().count().saturating_sub(1); // minus header
        let now = chrono_now();
        let tag = version.to_string();

        // Write signatures.csv
        let csv_path = self.repo_path.join("signatures.csv");
        fs::write(&csv_path, csv_data)?;

        // Write version metadata
        let meta = SignatureVersion {
            version: version.clone(),
            timestamp: now,
            author: author.to_string(),
            description: description.to_string(),
            commit_hash: String::new(), // filled after commit
            signature_count: sig_count,
        };
        let meta_path = self.repo_path.join("version.json");
        let meta_json = serde_json::to_string_pretty(&meta)
            .map_err(|e| StoreError::SerdeError(e.to_string()))?;
        fs::write(&meta_path, &meta_json)?;

        // Git add + commit + tag
        self.git(&["add", "signatures.csv", "version.json"])?;
        self.git(&["commit", "-m", &format!("push: {} — {}", tag, description)])?;
        self.git(&["tag", "-a", &tag, "-m", description])?;

        // Read back the commit hash
        let hash = self.git_output(&["rev-parse", "HEAD"])?;
        let mut committed = meta;
        committed.commit_hash = hash.trim().to_string();

        // Update version.json with the hash
        let updated_json = serde_json::to_string_pretty(&committed)
            .map_err(|e| StoreError::SerdeError(e.to_string()))?;
        fs::write(self.repo_path.join("version.json"), &updated_json)?;
        self.git(&["add", "version.json"])?;
        self.git(&[
            "commit",
            "--amend",
            "-m",
            &format!(
                "push: {} — {} [{}]",
                tag,
                description,
                &committed.commit_hash[..8]
            ),
        ])?;

        Ok(committed)
    }

    /// Pull (fetch + checkout) the latest signatures from the store.
    ///
    /// Returns the CSV data of the current HEAD.
    ///
    /// **Stability**: Stable since v1.2.0.
    pub fn pull(&self) -> Result<String, StoreError> {
        self.ensure_initialized()?;
        self.git(&["pull"])?;
        self.load_signatures()
    }

    /// Rollback to a specific version by checking out its git tag.
    ///
    /// **Stability**: Stable since v1.2.0.
    pub fn rollback(&self, version: &SemVer) -> Result<String, StoreError> {
        self.ensure_initialized()?;
        let tag = version.to_string();

        // Verify tag exists
        let tags = self.git_output(&["tag", "-l"])?;
        if !tags.lines().any(|t| t.trim() == tag) {
            return Err(StoreError::VersionNotFound(tag));
        }

        self.git(&["checkout", &tag])?;
        self.load_signatures()
    }

    /// List all versions (tags) in the store, newest first.
    ///
    /// **Stability**: Stable since v1.2.0.
    pub fn list(&self) -> Result<Vec<SignatureVersion>, StoreError> {
        self.ensure_initialized()?;

        let tags_output = self.git_output(&["tag", "-l", "--sort=-version:refname"])?;
        let mut versions = Vec::new();

        for tag in tags_output.lines() {
            let tag = tag.trim();
            if tag.is_empty() {
                continue;
            }

            // Try to read version.json at this tag
            let meta_json = self.git_output(&["show", &format!("{}:version.json", tag)])?;

            match serde_json::from_str::<SignatureVersion>(&meta_json) {
                Ok(mut ver) => {
                    if ver.commit_hash.is_empty() {
                        let hash = self.git_output(&["rev-list", "-n", "1", tag])?;
                        ver.commit_hash = hash.trim().to_string();
                    }
                    versions.push(ver);
                }
                Err(_) => continue,
            }
        }

        Ok(versions)
    }

    /// Get metadata for the current HEAD version.
    ///
    /// **Stability**: Stable since v1.2.0.
    pub fn current(&self) -> Result<SignatureVersion, StoreError> {
        self.ensure_initialized()?;
        let meta_path = self.repo_path.join("version.json");
        if !meta_path.exists() {
            return Err(StoreError::NotInitialized);
        }
        let data = fs::read_to_string(&meta_path)?;
        serde_json::from_str(&data).map_err(|e| StoreError::SerdeError(e.to_string()))
    }

    /// Load the current signatures CSV data.
    ///
    /// **Stability**: Stable since v1.2.0.
    pub fn load_signatures(&self) -> Result<String, StoreError> {
        let csv_path = self.repo_path.join("signatures.csv");
        if !csv_path.exists() {
            return Err(StoreError::NotInitialized);
        }
        Ok(fs::read_to_string(&csv_path)?)
    }

    /// Get the path to the store directory.
    pub fn path(&self) -> &Path {
        &self.repo_path
    }

    fn ensure_initialized(&self) -> Result<(), StoreError> {
        if !self.repo_path.join(".git").exists() {
            return Err(StoreError::NotInitialized);
        }
        Ok(())
    }

    fn git(&self, args: &[&str]) -> Result<(), StoreError> {
        let git_path = find_git()?;
        let output = Command::new(&git_path)
            .arg("-C")
            .arg(&self.repo_path)
            .args(args)
            .output()
            .map_err(|e| StoreError::GitError(format!("failed to run git: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(StoreError::GitError(stderr.trim().to_string()));
        }
        Ok(())
    }

    fn git_output(&self, args: &[&str]) -> Result<String, StoreError> {
        let git_path = find_git()?;
        let output = Command::new(&git_path)
            .arg("-C")
            .arg(&self.repo_path)
            .args(args)
            .output()
            .map_err(|e| StoreError::GitError(format!("failed to run git: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(StoreError::GitError(stderr.trim().to_string()));
        }
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
}

/// Find the git executable on the system.
fn find_git() -> Result<String, StoreError> {
    // Try PATH first
    if let Ok(output) = Command::new("git").arg("--version").output() {
        if output.status.success() {
            return Ok("git".to_string());
        }
    }
    // Try common Windows locations
    let candidates = [
        r"C:\Program Files\Git\bin\git.exe",
        r"C:\Program Files (x86)\Git\bin\git.exe",
    ];
    for path in &candidates {
        if std::path::Path::new(path).exists() {
            return Ok(path.to_string());
        }
    }
    Err(StoreError::GitError("git not found".to_string()))
}

/// Check if git is available on this system.
pub fn is_git_available() -> bool {
    find_git().is_ok()
}

/// Simple timestamp without pulling in chrono.
fn chrono_now() -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    format!(
        "{}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        1970 + (now / 31557600) as u32,
        ((now % 31557600) / 2592000) as u32 + 1,
        ((now % 2592000) / 86400) as u32 + 1,
        (now % 86400 / 3600) as u32,
        (now % 3600 / 60) as u32,
        (now % 60) as u32,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_store() -> (SignatureStore, tempfile::TempDir) {
        let dir = tempfile::TempDir::new().unwrap();
        let store = SignatureStore::new(dir.path().join("sigs"));
        (store, dir)
    }

    #[test]
    fn test_semver_parse() {
        let v = SemVer::parse("v1.2.3").unwrap();
        assert_eq!(
            v,
            SemVer {
                major: 1,
                minor: 2,
                patch: 3
            }
        );

        let v2 = SemVer::parse("1.0.0").unwrap();
        assert_eq!(v2, SemVer::new(1, 0, 0));

        assert!(SemVer::parse("bad").is_err());
        assert!(SemVer::parse("1.2").is_err());
    }

    #[test]
    fn test_semver_display() {
        let v = SemVer::new(1, 2, 3);
        assert_eq!(v.to_string(), "v1.2.3");
    }

    #[test]
    fn test_semver_ordering() {
        let v1 = SemVer::new(1, 0, 0);
        let v2 = SemVer::new(1, 1, 0);
        let v3 = SemVer::new(2, 0, 0);
        assert!(v1 < v2);
        assert!(v2 < v3);
    }

    #[test]
    fn test_init_creates_repo() {
        if !is_git_available() {
            return;
        }
        let (store, _dir) = temp_store();
        store.init().unwrap();
        assert!(store.repo_path.join(".git").exists());
        assert!(store.repo_path.join("README.md").exists());
    }

    #[test]
    fn test_push_and_pull() {
        if !is_git_available() {
            return;
        }
        let (store, _dir) = temp_store();
        store.init().unwrap();

        let csv = "category,protocol,signature,priority\nweb,HTTP/1.1,^HTTP/1\\.1\\s,100\n";
        let ver = store
            .push(csv, SemVer::new(1, 0, 0), "initial", "test")
            .unwrap();

        assert_eq!(ver.version, SemVer::new(1, 0, 0));
        assert_eq!(ver.signature_count, 1);
        assert!(!ver.commit_hash.is_empty());

        let pulled = store.load_signatures().unwrap();
        assert_eq!(pulled, csv);
    }

    #[test]
    fn test_list_versions() {
        if !is_git_available() {
            return;
        }
        let (store, _dir) = temp_store();
        store.init().unwrap();

        let csv1 = "category,protocol,signature,priority\nweb,HTTP,^HTTP,100\n";
        store
            .push(csv1, SemVer::new(1, 0, 0), "v1", "test")
            .unwrap();

        let csv2 = "category,protocol,signature,priority\nweb,HTTP/1.1,^HTTP/1\\.1\\s,100\n";
        store
            .push(csv2, SemVer::new(1, 1, 0), "v2", "test")
            .unwrap();

        let versions = store.list().unwrap();
        assert_eq!(versions.len(), 2);
        assert_eq!(versions[0].version, SemVer::new(1, 1, 0)); // newest first
        assert_eq!(versions[1].version, SemVer::new(1, 0, 0));
    }

    #[test]
    fn test_rollback() {
        if !is_git_available() {
            return;
        }
        let (store, _dir) = temp_store();
        store.init().unwrap();

        let csv1 = "category,protocol,signature,priority\nweb,HTTP,^HTTP,100\n";
        store
            .push(csv1, SemVer::new(1, 0, 0), "v1", "test")
            .unwrap();

        let csv2 = "category,protocol,signature,priority\nweb,HTTP/1.1,^HTTP/1\\.1\\s,100\n";
        store
            .push(csv2, SemVer::new(1, 1, 0), "v2", "test")
            .unwrap();

        // Rollback to v1.0.0
        let rolled = store.rollback(&SemVer::new(1, 0, 0)).unwrap();
        assert!(rolled.contains("web,HTTP,^HTTP,100"));
    }

    #[test]
    fn test_current_version() {
        if !is_git_available() {
            return;
        }
        let (store, _dir) = temp_store();
        store.init().unwrap();

        let csv = "category,protocol,signature,priority\nweb,HTTP,^HTTP,100\n";
        store
            .push(csv, SemVer::new(2, 3, 4), "test", "dev")
            .unwrap();

        let current = store.current().unwrap();
        assert_eq!(current.version, SemVer::new(2, 3, 4));
        assert_eq!(current.author, "dev");
    }

    #[test]
    fn test_not_initialized_errors() {
        let (store, _dir) = temp_store();
        assert!(store.pull().is_err());
        assert!(store.list().is_err());
        assert!(store.current().is_err());
    }
}
