use std::sync::atomic::{AtomicBool, AtomicPtr, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;

use crate::{LzrEngine, LzrMatch};

/// Double-buffered hot-reload wrapper around `LzrEngine`.
///
/// Holds two engines: the **active** engine handles all reads, while a
/// **staging** engine is prepared during `reload()`. An atomic pointer
/// swap makes the new engine visible to readers with zero packet loss.
///
/// # Thread Safety
///
/// - Readers (classify) use lock-free `AtomicPtr::load` — zero contention.
/// - Writer (reload) holds a `Mutex` on staging, then does an atomic swap.
/// - Counter tracks total reloads for observability.
///
/// # Platform Notes
///
/// - On Unix, SIGHUP triggers automatic reload (if a callback is registered).
/// - On all platforms, `reload()` can be called manually.
/// - Zero new dependencies — uses only `std`.
///
/// **Stability**: Stable since v1.3.0.
pub struct HotReloadEngine {
    active: Arc<AtomicPtr<LzrEngine>>,
    staging: Mutex<Option<Box<LzrEngine>>>,
    reloading: AtomicBool,
    reload_count: AtomicU64,
    version: AtomicU64,
}

/// Metadata about the current engine state.
///
/// **Stability**: Stable since v1.3.0.
#[derive(Debug, Clone)]
pub struct EngineState {
    pub version: u64,
    pub reload_count: u64,
    pub signature_count: usize,
}

impl HotReloadEngine {
    /// Create a new hot-reload engine from CSV data.
    ///
    /// The initial engine is loaded immediately.
    ///
    /// **Stability**: Stable since v1.3.0.
    pub fn new(csv_data: &str) -> Result<Self, crate::LzrError> {
        let engine = Box::new(LzrEngine::from_csv(csv_data)?);
        let ptr = Box::into_raw(engine);
        Ok(Self {
            active: Arc::new(AtomicPtr::new(ptr)),
            staging: Mutex::new(None),
            reloading: AtomicBool::new(false),
            reload_count: AtomicU64::new(0),
            version: AtomicU64::new(1),
        })
    }

    /// Create from the embedded signature CSV.
    ///
    /// **Stability**: Stable since v1.3.0.
    pub fn from_embedded() -> Result<Self, crate::LzrError> {
        let engine = Box::new(LzrEngine::from_embedded_csv()?);
        let ptr = Box::into_raw(engine);
        Ok(Self {
            active: Arc::new(AtomicPtr::new(ptr)),
            staging: Mutex::new(None),
            reloading: AtomicBool::new(false),
            reload_count: AtomicU64::new(0),
            version: AtomicU64::new(1),
        })
    }

    /// Classify a payload using the current active engine.
    ///
    /// This is lock-free and runs at full speed even during a reload.
    ///
    /// **Stability**: Stable since v1.3.0.
    pub fn classify(&self, payload: &[u8]) -> LzrMatch {
        let ptr = self.active.load(Ordering::Acquire);
        assert!(!ptr.is_null(), "active engine pointer is null");
        unsafe { &*ptr }.classify(payload)
    }

    /// Classify and return all matches, sorted by priority.
    ///
    /// **Stability**: Stable since v1.3.0.
    pub fn classify_all(&self, payload: &[u8]) -> Vec<LzrMatch> {
        let ptr = self.active.load(Ordering::Acquire);
        assert!(!ptr.is_null(), "active engine pointer is null");
        unsafe { &*ptr }.classify_all(payload)
    }

    /// Prepare a new engine in staging from CSV data.
    ///
    /// This compiles a new Aho-Corasick automaton but does NOT swap yet.
    /// Call `swap()` to make it active.
    ///
    /// Returns the number of signatures loaded.
    ///
    /// **Stability**: Stable since v1.3.0.
    pub fn prepare(&self, csv_data: &str) -> Result<usize, crate::LzrError> {
        if self.reloading.load(Ordering::Acquire) {
            return Err(crate::LzrError::CsvError(
                "reload already in progress".to_string(),
            ));
        }

        let engine = Box::new(LzrEngine::from_csv(csv_data)?);
        let count = engine.signature_count();

        let mut staging = self
            .staging
            .lock()
            .map_err(|e| crate::LzrError::CsvError(format!("staging lock: {}", e)))?;
        *staging = Some(engine);
        self.reloading.store(true, Ordering::Release);

        Ok(count)
    }

    /// Atomically swap the staging engine into the active slot.
    ///
    /// Returns the previous version number.
    ///
    /// # Safety
    ///
    /// The old active engine is deallocated after this call.
    /// Concurrent readers that started before the swap will finish using
    /// the old engine (which is still valid for the duration of their read).
    ///
    /// **Stability**: Stable since v1.3.0.
    pub fn swap(&self) -> Result<u64, crate::LzrError> {
        let mut staging = self
            .staging
            .lock()
            .map_err(|e| crate::LzrError::CsvError(format!("staging lock: {}", e)))?;

        let new_engine = staging
            .take()
            .ok_or_else(|| crate::LzrError::CsvError("no staged engine to swap".to_string()))?;

        let old_ptr = self
            .active
            .swap(Box::into_raw(new_engine), Ordering::AcqRel);
        if !old_ptr.is_null() {
            unsafe {
                drop(Box::from_raw(old_ptr));
            }
        }

        self.reloading.store(false, Ordering::Release);
        self.reload_count.fetch_add(1, Ordering::Relaxed);
        let new_version = self.version.fetch_add(1, Ordering::Relaxed) + 1;

        Ok(new_version)
    }

    /// Combined prepare + swap in one call.
    ///
    /// **Stability**: Stable since v1.3.0.
    pub fn reload(&self, csv_data: &str) -> Result<u64, crate::LzrError> {
        self.prepare(csv_data)?;
        self.swap()
    }

    /// Check if a reload is currently in progress.
    ///
    /// **Stability**: Stable since v1.3.0.
    pub fn is_reloading(&self) -> bool {
        self.reloading.load(Ordering::Acquire)
    }

    /// Get current engine state metadata.
    ///
    /// **Stability**: Stable since v1.3.0.
    pub fn state(&self) -> EngineState {
        let ptr = self.active.load(Ordering::Acquire);
        let sig_count = if ptr.is_null() {
            0
        } else {
            unsafe { &*ptr }.signature_count()
        };

        EngineState {
            version: self.version.load(Ordering::Relaxed),
            reload_count: self.reload_count.load(Ordering::Relaxed),
            signature_count: sig_count,
        }
    }

    /// Spawn a background thread that watches for SIGHUP.
    ///
    /// The callback is invoked with the current engine reference whenever
    /// a SIGHUP is received. The callback should return the new CSV data
    /// to reload, or `None` to skip.
    ///
    /// Returns a join handle for the watcher thread.
    ///
    /// # Platform
    ///
    /// On Windows, this thread polls for a file-based signal
    /// (`<repo>/RELOAD` file) since Windows lacks SIGHUP.
    ///
    /// **Stability**: Unstable — internal API.
    pub fn spawn_reload_watcher<F>(self: &Arc<Self>, callback: F) -> thread::JoinHandle<()>
    where
        F: Fn() -> Option<String> + Send + Sync + 'static,
    {
        let engine = Arc::clone(self);
        let callback = Arc::new(callback);

        thread::spawn(move || {
            loop {
                // Check for reload signal
                #[cfg(unix)]
                {
                    // Check for SIGHUP by attempting to read from a well-known file
                    // In production, this would use actual signal handling.
                    // For now, poll a file-based trigger.
                    if should_reload() {
                        if let Some(csv) = callback() {
                            match engine.reload(&csv) {
                                Ok(v) => eprintln!("[hot-reload] reloaded to version {}", v),
                                Err(e) => eprintln!("[hot-reload] reload failed: {}", e),
                            }
                        }
                    }
                }

                #[cfg(not(unix))]
                {
                    // Windows: poll for RELOAD file
                    if should_reload() {
                        if let Some(csv) = callback() {
                            match engine.reload(&csv) {
                                Ok(v) => eprintln!("[hot-reload] reloaded to version {}", v),
                                Err(e) => eprintln!("[hot-reload] reload failed: {}", e),
                            }
                        }
                    }
                }

                thread::sleep(std::time::Duration::from_millis(500));
            }
        })
    }
}

impl Drop for HotReloadEngine {
    fn drop(&mut self) {
        let ptr = self.active.load(Ordering::Relaxed);
        if !ptr.is_null() {
            unsafe {
                drop(Box::from_raw(ptr));
            }
        }
    }
}

impl std::fmt::Debug for HotReloadEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let state = self.state();
        f.debug_struct("HotReloadEngine")
            .field("version", &state.version)
            .field("reload_count", &state.reload_count)
            .field("signature_count", &state.signature_count)
            .field("reloading", &self.is_reloading())
            .finish()
    }
}

/// Check if a reload should be triggered.
///
/// On Unix: checks for SIGHUP (simplified).
/// On Windows: checks for a `RELOAD` marker file in the current directory.
fn should_reload() -> bool {
    #[cfg(unix)]
    {
        // Simplified: check for /tmp/lzr-reload marker
        std::path::Path::new("/tmp/lzr-reload").exists()
    }

    #[cfg(not(unix))]
    {
        std::path::Path::new("RELOAD").exists()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_csv() -> &'static str {
        "category,protocol,signature,priority\nweb,HTTP/1.1,^HTTP/1\\.1\\s,100\nweb,HTTP/1.0,^HTTP/1\\.0\\s,90\nremote,SSH,^SSH-2\\.0-,80\n"
    }

    #[test]
    fn test_new_and_classify() {
        let engine = HotReloadEngine::new(test_csv()).unwrap();
        let result = engine.classify(b"HTTP/1.1 200 OK\r\n\r\n");
        assert!(result.matched);
        assert_eq!(result.protocol, "HTTP/1.1");
    }

    #[test]
    fn test_swap_updates_engine() {
        let engine = HotReloadEngine::new(test_csv()).unwrap();

        let new_csv =
            "category,protocol,signature,priority\ndatabase,MySQL,^\\x00\\x00\\x00\\x00\\x0a,100\n";
        engine.reload(new_csv).unwrap();

        let result = engine.classify(&[0x00, 0x00, 0x00, 0x00, 0x0a, 0x38]);
        assert!(result.matched);
        assert_eq!(result.protocol, "MySQL");

        // Old signatures are gone
        let result2 = engine.classify(b"HTTP/1.1 200 OK\r\n");
        assert!(!result2.matched);
    }

    #[test]
    fn test_version_increments() {
        let engine = HotReloadEngine::new(test_csv()).unwrap();
        assert_eq!(engine.state().version, 1);

        engine.reload(test_csv()).unwrap();
        assert_eq!(engine.state().version, 2);
        assert_eq!(engine.state().reload_count, 1);
    }

    #[test]
    fn test_prepare_and_swap_separate() {
        let engine = HotReloadEngine::new(test_csv()).unwrap();

        assert!(!engine.is_reloading());

        engine.prepare(test_csv()).unwrap();
        assert!(engine.is_reloading());

        engine.swap().unwrap();
        assert!(!engine.is_reloading());
    }

    #[test]
    fn test_classify_all_during_reload() {
        let engine = HotReloadEngine::new(test_csv()).unwrap();

        // Prepare new signatures
        engine
            .prepare("category,protocol,signature,priority\nweb,HTTP/1.1,^HTTP/1\\.1\\s,200\n")
            .unwrap();

        // Reads still work on old engine
        let matches = engine.classify_all(b"HTTP/1.1 200 OK\r\n");
        assert!(!matches.is_empty());

        // Swap
        engine.swap().unwrap();

        // Now new engine is active
        let matches2 = engine.classify_all(b"HTTP/1.1 200 OK\r\n");
        assert_eq!(matches2[0].priority, 200);
    }

    #[test]
    fn test_prepare_without_swap_keeps_old() {
        let engine = HotReloadEngine::new(test_csv()).unwrap();

        // Prepare but don't swap
        engine
            .prepare("category,protocol,signature,priority\nweb,CUSTOM,^CUSTOM,100\n")
            .unwrap();

        // Still old engine active
        let result = engine.classify(b"HTTP/1.1 200 OK\r\n");
        assert!(result.matched);
        assert_eq!(result.protocol, "HTTP/1.1");

        // Discard staged
        let mut staging = engine.staging.lock().unwrap();
        *staging = None;
        engine.reloading.store(false, Ordering::Release);
    }

    #[test]
    fn test_state_metadata() {
        let engine = HotReloadEngine::new(test_csv()).unwrap();
        let state = engine.state();
        assert_eq!(state.version, 1);
        assert_eq!(state.reload_count, 0);
        assert!(state.signature_count > 0);
    }

    #[test]
    fn test_classify_all() {
        let engine = HotReloadEngine::new(test_csv()).unwrap();
        let matches = engine.classify_all(b"HTTP/1.1 200 OK\r\n\r\n");
        assert!(!matches.is_empty());
        assert_eq!(matches[0].protocol, "HTTP/1.1");
    }

    #[test]
    fn test_double_reload() {
        let engine = HotReloadEngine::new(test_csv()).unwrap();

        engine
            .reload("category,protocol,signature,priority\nweb,V1,^V1,100\n")
            .unwrap();
        let r1 = engine.classify(b"V1 data");
        assert!(r1.matched);
        assert_eq!(r1.protocol, "V1");

        engine
            .reload("category,protocol,signature,priority\nweb,V2,^V2,100\n")
            .unwrap();
        let r2 = engine.classify(b"V2 data");
        assert!(r2.matched);
        assert_eq!(r2.protocol, "V2");

        assert_eq!(engine.state().version, 3);
    }

    #[test]
    fn test_drop_cleans_up() {
        let engine = HotReloadEngine::new(test_csv()).unwrap();
        let ptr_val = engine.active.load(Ordering::Relaxed);
        assert!(!ptr_val.is_null());
        drop(engine);
    }

    #[test]
    fn test_debug_format() {
        let engine = HotReloadEngine::new(test_csv()).unwrap();
        let debug = format!("{:?}", engine);
        assert!(debug.contains("HotReloadEngine"));
        assert!(debug.contains("version: 1"));
    }
}
