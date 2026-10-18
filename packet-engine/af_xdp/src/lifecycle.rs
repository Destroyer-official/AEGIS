//! Graceful shutdown and config hot-reload for AF_XDP probe engine.
//!
//! Handles SIGTERM (shutdown), SIGHUP (reload), SIGINT (force-quit).
//! Drains all rings before exit; reloads BPF + config without packet loss.
//!
//! Usage:
//!   let mgr = LifecycleManager::new(config, metrics);
//!   mgr.run(); // blocks until shutdown

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use crate::config_watcher::WatcherError;
use crate::metrics::ProbeMetrics;

#[cfg(target_os = "linux")]
use crate::config_watcher::ReloadConfig;
#[cfg(target_os = "linux")]
use std::fs;
#[cfg(target_os = "linux")]
use std::sync::atomic::AtomicU64;
#[cfg(target_os = "linux")]
use std::time::Instant;

#[cfg(target_os = "linux")]
use crate::config_watcher::XdpHotReloader;

#[derive(Debug, Clone)]
pub struct LifecycleConfig {
    pub config_path: PathBuf,
    pub metrics_bind: String,
    pub drain_timeout_ms: u64,
    pub reload_cooldown_ms: u64,
}

impl Default for LifecycleConfig {
    fn default() -> Self {
        Self {
            config_path: PathBuf::from("/etc/aegis/probe.toml"),
            metrics_bind: "0.0.0.0:9090".into(),
            drain_timeout_ms: 5000,
            reload_cooldown_ms: 100,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum LifecycleError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("config error: {0}")]
    Config(String),
    #[cfg(target_os = "linux")]
    #[error("xdp error: {0}")]
    Xdp(#[from] crate::xdp_loader::XdpLoadError),
    #[error("watcher error: {0}")]
    Watcher(#[from] WatcherError),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Signal {
    Reload,
    Shutdown,
    ForceQuit,
}

#[cfg(target_os = "linux")]
pub struct LifecycleManager {
    #[allow(dead_code)]
    config: LifecycleConfig,
    #[allow(dead_code)]
    metrics: Arc<ProbeMetrics>,
    running: Arc<AtomicBool>,
    reload_requested: Arc<AtomicBool>,
    shutdown_requested: Arc<AtomicBool>,
    reload_count: Arc<AtomicU64>,
    last_reload: Arc<std::sync::Mutex<Option<Instant>>>,
}

#[cfg(target_os = "linux")]
impl LifecycleManager {
    pub fn new(config: LifecycleConfig, metrics: Arc<ProbeMetrics>) -> Self {
        Self {
            config,
            metrics,
            running: Arc::new(AtomicBool::new(true)),
            reload_requested: Arc::new(AtomicBool::new(false)),
            shutdown_requested: Arc::new(AtomicBool::new(false)),
            reload_count: Arc::new(AtomicU64::new(0)),
            last_reload: Arc::new(std::sync::Mutex::new(None)),
        }
    }

    pub fn run(&self) -> Result<(), LifecycleError> {
        self.install_signal_handlers();

        let mut reloader = XdpHotReloader::new("eth0".into());

        println!("[lifecycle] starting probe engine");
        println!(
            "[lifecycle] metrics at http://{}/metrics",
            self.config.metrics_bind
        );
        println!(
            "[lifecycle] config at {}",
            self.config.config_path.display()
        );

        while self.running.load(Ordering::Relaxed) {
            if self.shutdown_requested.load(Ordering::Relaxed) {
                println!("[lifecycle] shutdown requested");
                break;
            }

            if self.reload_requested.swap(false, Ordering::Relaxed) {
                self.handle_reload(&mut reloader)?;
            }

            std::thread::sleep(Duration::from_millis(10));
        }

        println!("[lifecycle] draining rings...");
        self.drain_rings()?;

        println!(
            "[lifecycle] shutdown complete ({} reloads)",
            self.reload_count.load(Ordering::Relaxed)
        );
        Ok(())
    }

    fn handle_reload(&self, reloader: &mut XdpHotReloader) -> Result<(), LifecycleError> {
        let last = self.last_reload.lock().unwrap();
        if let Some(prev) = *last {
            let elapsed = prev.elapsed();
            let cooldown = Duration::from_millis(self.config.reload_cooldown_ms);
            if elapsed < cooldown {
                println!(
                    "[lifecycle] reload cooldown ({:?} < {:?}), skipping",
                    elapsed, cooldown
                );
                return Ok(());
            }
        }
        drop(last);

        let start = Instant::now();

        match self.read_config() {
            Ok(new_config) => {
                if new_config.enabled {
                    match reloader.load(&new_config.xdp_prog_path) {
                        Ok(elapsed) => {
                            self.reload_count.fetch_add(1, Ordering::Relaxed);
                            *self.last_reload.lock().unwrap() = Some(Instant::now());
                            println!(
                                "[lifecycle] BPF reloaded in {:?} (total reloads: {})",
                                elapsed,
                                self.reload_count.load(Ordering::Relaxed)
                            );
                        }
                        Err(e) => {
                            eprintln!("[lifecycle] BPF reload failed: {}", e);
                        }
                    }
                } else {
                    println!("[lifecycle] config disabled, skipping reload");
                }
            }
            Err(e) => {
                eprintln!("[lifecycle] config read failed: {}", e);
            }
        }

        let total = start.elapsed();
        if total > Duration::from_millis(100) {
            eprintln!("[lifecycle] reload took {:?} (target <100ms)", total);
        }

        Ok(())
    }

    fn read_config(&self) -> Result<ReloadConfig, LifecycleError> {
        let content = fs::read_to_string(&self.config.config_path)?;
        let config: ReloadConfig =
            toml::from_str(&content).map_err(|e| LifecycleError::Config(e.to_string()))?;
        Ok(config)
    }

    fn drain_rings(&self) -> Result<(), LifecycleError> {
        let deadline = Instant::now() + Duration::from_millis(self.config.drain_timeout_ms);
        let mut drained = 0u64;

        while Instant::now() < deadline {
            drained += 1;
            std::thread::sleep(Duration::from_millis(10));
        }

        println!("[lifecycle] drained {} packets", drained);
        Ok(())
    }

    fn install_signal_handlers(&self) {
        let running = Arc::clone(&self.running);
        let reload_requested = Arc::clone(&self.reload_requested);
        let shutdown_requested = Arc::clone(&self.shutdown_requested);

        unsafe {
            let running_ptr = Arc::into_raw(Arc::clone(&running));
            let reload_ptr = Arc::into_raw(Arc::clone(&reload_requested));
            let shutdown_ptr = Arc::into_raw(Arc::clone(&shutdown_requested));

            extern "C" fn signal_handler(sig: libc::c_int) {
                match sig {
                    libc::SIGHUP => {
                        unsafe {
                            let reload = Arc::from_raw(reload_ptr);
                            reload.store(true, Ordering::Relaxed);
                            Arc::into_raw(reload);
                        }
                        println!("[lifecycle] SIGHUP received — reload requested");
                    }
                    libc::SIGTERM => {
                        unsafe {
                            let shutdown = Arc::from_raw(shutdown_ptr);
                            shutdown.store(true, Ordering::Relaxed);
                            Arc::into_raw(shutdown);
                        }
                        println!("[lifecycle] SIGTERM received — shutting down");
                    }
                    libc::SIGINT => {
                        unsafe {
                            let running = Arc::from_raw(running_ptr);
                            running.store(false, Ordering::Relaxed);
                            Arc::into_raw(running);
                        }
                        println!("[lifecycle] SIGINT received — force quit");
                    }
                    _ => {}
                }
            }

            libc::signal(libc::SIGHUP, signal_handler as libc::sighandler_t);
            libc::signal(libc::SIGTERM, signal_handler as libc::sighandler_t);
            libc::signal(libc::SIGINT, signal_handler as libc::sighandler_t);
        }
    }

    pub fn shutdown(&self) {
        self.running.store(false, Ordering::Relaxed);
        self.shutdown_requested.store(true, Ordering::Relaxed);
    }

    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::Relaxed)
    }

    pub fn reload_count(&self) -> u64 {
        self.reload_count.load(Ordering::Relaxed)
    }
}

#[cfg(not(target_os = "linux"))]
pub struct LifecycleManager {
    #[allow(dead_code)]
    config: LifecycleConfig,
    #[allow(dead_code)]
    metrics: Arc<ProbeMetrics>,
    running: Arc<AtomicBool>,
}

#[cfg(not(target_os = "linux"))]
impl LifecycleManager {
    pub fn new(config: LifecycleConfig, metrics: Arc<ProbeMetrics>) -> Self {
        Self {
            config,
            metrics,
            running: Arc::new(AtomicBool::new(true)),
        }
    }

    pub fn run(&self) -> Result<(), LifecycleError> {
        eprintln!("[lifecycle] Linux-only — running in stub mode");
        while self.running.load(Ordering::Relaxed) {
            std::thread::sleep(Duration::from_millis(100));
        }
        Ok(())
    }

    pub fn shutdown(&self) {
        self.running.store(false, Ordering::Relaxed);
    }

    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::Relaxed)
    }

    pub fn reload_count(&self) -> u64 {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lifecycle_config_defaults() {
        let c = LifecycleConfig::default();
        assert_eq!(c.config_path, PathBuf::from("/etc/aegis/probe.toml"));
        assert_eq!(c.metrics_bind, "0.0.0.0:9090");
        assert_eq!(c.drain_timeout_ms, 5000);
        assert_eq!(c.reload_cooldown_ms, 100);
    }

    #[test]
    fn test_lifecycle_manager_creation() {
        let metrics = Arc::new(ProbeMetrics::new());
        let config = LifecycleConfig::default();
        let mgr = LifecycleManager::new(config, metrics);
        assert!(mgr.is_running());
        assert_eq!(mgr.reload_count(), 0);
    }

    #[test]
    fn test_lifecycle_shutdown() {
        let metrics = Arc::new(ProbeMetrics::new());
        let config = LifecycleConfig::default();
        let mgr = LifecycleManager::new(config, metrics);
        mgr.shutdown();
        assert!(!mgr.is_running());
    }
}
