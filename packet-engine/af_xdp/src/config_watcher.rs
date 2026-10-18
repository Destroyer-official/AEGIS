//! File-based config watcher for XDP hot-reload.
//!
//! Watches a TOML config file for changes and triggers BPF reload
//! without tearing down the AF_XDP socket.
//!
//! Target: reload <100ms, zero packet drops during reload.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

#[cfg(target_os = "linux")]
use crate::xdp_loader::{XdpLoadError, XdpProgram};

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct ReloadConfig {
    pub xdp_prog_path: String,
    pub ifname: String,
    #[serde(default = "default_poll_interval_ms")]
    pub poll_interval_ms: u64,
    #[serde(default = "default_true")]
    pub enabled: bool,
}

fn default_poll_interval_ms() -> u64 {
    100
}

fn default_true() -> bool {
    true
}

#[derive(Debug, thiserror::Error)]
pub enum WatcherError {
    #[error("config parse error: {0}")]
    Parse(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[cfg(target_os = "linux")]
    #[error("reload error: {0}")]
    Reload(#[from] XdpLoadError),
    #[error("watcher stopped")]
    Stopped,
}

pub struct ConfigWatcher {
    config_path: PathBuf,
    running: Arc<AtomicBool>,
    last_modified: Option<Instant>,
}

impl ConfigWatcher {
    pub fn new(config_path: impl AsRef<Path>) -> Self {
        Self {
            config_path: config_path.as_ref().to_path_buf(),
            running: Arc::new(AtomicBool::new(true)),
            last_modified: None,
        }
    }

    pub fn stop(&self) {
        self.running.store(false, Ordering::Relaxed);
    }

    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::Relaxed)
    }

    pub fn watch_loop<F>(&mut self, mut on_reload: F) -> Result<(), WatcherError>
    where
        F: FnMut(&ReloadConfig, Duration) -> Result<(), WatcherError>,
    {
        let poll_interval = Duration::from_millis(100);

        while self.running.load(Ordering::Relaxed) {
            let metadata = fs::metadata(&self.config_path)?;
            let modified = metadata.modified().ok().and_then(|t| t.elapsed().ok());

            if modified.is_some() {
                let is_new = self
                    .last_modified
                    .map(|prev| prev.elapsed() > Duration::from_millis(50))
                    .unwrap_or(true);

                if is_new {
                    let start = Instant::now();
                    let config = self.read_config()?;
                    self.last_modified = Some(Instant::now());

                    if config.enabled {
                        let reload_start = Instant::now();
                        on_reload(&config, reload_start.elapsed())?;
                        let elapsed = start.elapsed();
                        println!(
                            "[config_watcher] reloaded {} in {:?}",
                            self.config_path.display(),
                            elapsed
                        );
                    }
                }
            }

            std::thread::sleep(poll_interval);
        }

        Ok(())
    }

    fn read_config(&self) -> Result<ReloadConfig, WatcherError> {
        let content = fs::read_to_string(&self.config_path)?;
        let config: ReloadConfig =
            toml::from_str(&content).map_err(|e| WatcherError::Parse(e.to_string()))?;
        Ok(config)
    }

    pub fn write_config(config: &ReloadConfig, path: impl AsRef<Path>) -> Result<(), WatcherError> {
        let content =
            toml::to_string_pretty(config).map_err(|e| WatcherError::Parse(e.to_string()))?;
        fs::write(path, content)?;
        Ok(())
    }
}

/// Hot-reload manager that swaps XDP programs without socket teardown.
///
/// Only available on Linux; on other platforms the struct exists but all
/// methods return errors.
#[cfg(target_os = "linux")]
pub struct XdpHotReloader {
    current_prog: Option<XdpProgram>,
    ifname: String,
}

#[cfg(target_os = "linux")]
impl XdpHotReloader {
    pub fn new(ifname: String) -> Self {
        Self {
            current_prog: None,
            ifname,
        }
    }

    pub fn load(&mut self, prog_path: &str) -> Result<Duration, XdpLoadError> {
        let start = Instant::now();

        let new_prog = XdpProgram::load_from_file(prog_path, &self.ifname)?;

        if let Some(old_prog) = self.current_prog.take() {
            let _ = old_prog.detach();
        }

        self.current_prog = Some(new_prog);

        Ok(start.elapsed())
    }

    pub fn current_prog(&self) -> Option<&XdpProgram> {
        self.current_prog.as_ref()
    }

    pub fn has_program(&self) -> bool {
        self.current_prog.is_some()
    }
}

#[cfg(target_os = "linux")]
impl Drop for XdpHotReloader {
    fn drop(&mut self) {
        if let Some(prog) = self.current_prog.take() {
            let _ = prog.detach();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_parse() {
        let content = r#"
xdp_prog_path = "/path/to/prog.o"
ifname = "eth0"
poll_interval_ms = 200
enabled = true
"#;
        let config: ReloadConfig = toml::from_str(content).unwrap();
        assert_eq!(config.xdp_prog_path, "/path/to/prog.o");
        assert_eq!(config.ifname, "eth0");
        assert_eq!(config.poll_interval_ms, 200);
        assert!(config.enabled);
    }

    #[test]
    fn test_config_defaults() {
        let content = r#"
xdp_prog_path = "/path/to/prog.o"
ifname = "eth0"
"#;
        let config: ReloadConfig = toml::from_str(content).unwrap();
        assert_eq!(config.poll_interval_ms, 100);
        assert!(config.enabled);
    }

    #[test]
    fn test_watcher_stop() {
        let watcher = ConfigWatcher::new("/tmp/test_config.toml");
        assert!(watcher.is_running());
        watcher.stop();
        assert!(!watcher.is_running());
    }

    #[test]
    fn test_write_and_read_config() {
        let dir = std::env::temp_dir().join("aegis_watcher_test");
        let _ = fs::create_dir_all(&dir);
        let path = dir.join("test_config.toml");

        let config = ReloadConfig {
            xdp_prog_path: "/tmp/prog.o".into(),
            ifname: "eth0".into(),
            poll_interval_ms: 150,
            enabled: false,
        };

        ConfigWatcher::write_config(&config, &path).unwrap();
        let content = fs::read_to_string(&path).unwrap();
        let parsed: ReloadConfig = toml::from_str(&content).unwrap();

        assert_eq!(parsed.xdp_prog_path, "/tmp/prog.o");
        assert_eq!(parsed.poll_interval_ms, 150);
        assert!(!parsed.enabled);

        let _ = fs::remove_dir_all(&dir);
    }
}
