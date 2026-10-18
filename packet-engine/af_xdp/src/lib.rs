pub mod config_watcher;
pub mod lifecycle;
pub mod metrics;
pub mod numa;
pub mod parse;
pub mod probe;
pub mod rings;
mod umem;

#[cfg(target_os = "linux")]
mod socket;
#[cfg(target_os = "linux")]
mod xdp_loader;

#[cfg(target_os = "linux")]
pub use config_watcher::XdpHotReloader;
pub use config_watcher::{ConfigWatcher, ReloadConfig, WatcherError};
pub use parse::{ParseError, ParsedPacket, Protocol};
pub use probe::{
    build_syn_packet, validate_syn_ack, PipelinedTracker, ProbeEncoder, ProbeTarget,
    ShardedProbeEngine,
};
pub use rings::xdp_desc;
pub use umem::{Umem, UmemConfig, UmemError};

pub use lifecycle::{LifecycleConfig, LifecycleError, LifecycleManager, Signal};
pub use metrics::{MetricsServer, ProbeMetrics};
pub use numa::{NumaError, NumaNode, NumaPinning, QueueInfo};

#[cfg(target_os = "linux")]
pub use socket::{XskConfig, XskError, XskSocket};
#[cfg(target_os = "linux")]
pub use xdp_loader::{XdpLoadError, XdpProgram};

#[cfg(target_os = "linux")]
pub use rings::{COMPLETION_RING, FILL_RING, RX_RING, TX_RING};

pub const FRAME_SIZE: usize = 2048;
pub const HEADROOM: usize = 256;
pub const FRAME_DATA_SIZE: usize = FRAME_SIZE - HEADROOM;

#[cfg(target_os = "linux")]
pub struct AfXdpEngine {
    socket: XskSocket,
    fill_ring: *mut FILL_RING,
    completion_ring: *mut COMPLETION_RING,
    rx_ring: *mut RX_RING,
    tx_ring: *mut TX_RING,
    umem: Umem,
}

#[cfg(target_os = "linux")]
unsafe impl Send for AfXdpEngine {}

#[derive(Debug, thiserror::Error)]
pub enum EngineError {
    #[error("UMEM error: {0}")]
    Umem(#[from] UmemError),
    #[cfg(target_os = "linux")]
    #[error("socket error: {0}")]
    Socket(#[from] XskError),
    #[cfg(target_os = "linux")]
    #[error("XDP load error: {0}")]
    Xdp(#[from] XdpLoadError),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

#[cfg(target_os = "linux")]
impl AfXdpEngine {
    pub fn new(ifname: &str, config: &XskConfig) -> Result<Self, EngineError> {
        let umem_config = UmemConfig {
            frame_count: config.frame_count,
            ..Default::default()
        };
        let umem = Umem::new(&umem_config)?;
        let socket = XskSocket::new(ifname, &umem, config)?;

        let engine = Self {
            fill_ring: socket.fill_ring(),
            completion_ring: socket.completion_ring(),
            rx_ring: socket.rx_ring(),
            tx_ring: socket.tx_ring(),
            socket,
            umem,
        };

        Ok(engine)
    }

    pub fn load_xdp_program(&self, path: &str, ifname: &str) -> Result<XdpProgram, EngineError> {
        Ok(XdpProgram::load_from_file(path, ifname)?)
    }

    pub fn refill_fill_ring(&mut self) -> Result<u32, EngineError> {
        let available = unsafe { (*self.fill_ring).available() };
        let mut added = 0u32;

        for _ in 0..available {
            match self.umem.next_free_frame_addr() {
                Ok(addr) => unsafe {
                    let produced = (*self.fill_ring).produce(&[addr]);
                    if produced > 0 {
                        added += produced;
                    } else {
                        self.umem.return_frame_addr(addr);
                        break;
                    }
                },
                Err(_) => break,
            }
        }

        Ok(added)
    }

    pub fn refill_fill_ring_batch(&mut self, count: u32) -> Result<u32, EngineError> {
        let available = unsafe { (*self.fill_ring).available() };
        let to_add = count.min(available);
        let mut added = 0u32;

        for _ in 0..to_add {
            match self.umem.next_free_frame_addr() {
                Ok(addr) => unsafe {
                    let produced = (*self.fill_ring).produce(&[addr]);
                    if produced > 0 {
                        added += produced;
                    } else {
                        self.umem.return_frame_addr(addr);
                        break;
                    }
                },
                Err(_) => break,
            }
        }

        Ok(added)
    }

    pub fn recv(&mut self, packets: &mut Vec<ParsedPacket>, max: u32) -> Result<u32, EngineError> {
        let rx_descs = unsafe { (*self.rx_ring).consume_batch(max) };
        let count = rx_descs.len() as u32;

        for desc in &rx_descs {
            let pkt_data = unsafe {
                let base = self.umem.frame_base_ptr(desc.addr);
                std::slice::from_raw_parts(base.add(HEADROOM), desc.len as usize)
            };

            if let Ok(parsed) = ParsedPacket::parse(pkt_data) {
                packets.push(parsed);
            }
        }

        if !rx_descs.is_empty() {
            let frame_addrs: Vec<u64> = rx_descs.iter().map(|d| d.addr).collect();
            unsafe {
                (*self.fill_ring).produce(&frame_addrs);
            }
        }

        Ok(count)
    }

    pub fn recv_raw(&mut self, max: u32) -> Result<Vec<RawPacket>, EngineError> {
        let rx_descs = unsafe { (*self.rx_ring).consume_batch(max) };
        let mut raw_packets = Vec::with_capacity(rx_descs.len());

        for desc in &rx_descs {
            let pkt_data = unsafe {
                let base = self.umem.frame_base_ptr(desc.addr);
                std::slice::from_raw_parts(base.add(HEADROOM), desc.len as usize)
            };

            raw_packets.push(RawPacket {
                data: pkt_data.to_vec(),
                addr: desc.addr,
            });
        }

        if !rx_descs.is_empty() {
            let frame_addrs: Vec<u64> = rx_descs.iter().map(|d| d.addr).collect();
            unsafe {
                (*self.fill_ring).produce(&frame_addrs);
            }
        }

        Ok(raw_packets)
    }

    pub fn send(&mut self, descs: &[xdp_desc]) -> Result<u32, EngineError> {
        let mut sent = 0u32;
        let mut offset = 0;

        while offset < descs.len() {
            let batch = &descs[offset..];
            unsafe {
                let produced = (*self.tx_ring).produce(batch);
                if produced == 0 {
                    break;
                }
                sent += produced;
                offset += produced as usize;
            }
        }

        Ok(sent)
    }

    pub fn send_frames(&mut self, frame_addrs: &[u64], pkt_len: u32) -> Result<u32, EngineError> {
        let descs: Vec<xdp_desc> = frame_addrs
            .iter()
            .map(|addr| xdp_desc {
                addr: *addr,
                len: pkt_len,
                options: 0,
            })
            .collect();

        self.send(&descs)
    }

    pub fn poll_rx(&self, timeout_ms: i32) -> Result<bool, EngineError> {
        use nix::poll::{poll, PollFd, PollFlags};

        let fds = [PollFd::new(self.socket.rx_fd(), PollFlags::POLLIN)];
        let n = poll(&fds, timeout_ms).map_err(|e| std::io::Error::from(e))?;

        Ok(n > 0
            && (fds[0].revents().unwrap_or(PollFlags::empty()) & PollFlags::POLLIN)
                != PollFlags::empty())
    }

    pub fn poll_tx(&self, timeout_ms: i32) -> Result<bool, EngineError> {
        use nix::poll::{poll, PollFd, PollFlags};

        let fds = [PollFd::new(self.socket.tx_fd(), PollFlags::POLLOUT)];
        let n = poll(&fds, timeout_ms).map_err(|e| std::io::Error::from(e))?;

        Ok(n > 0
            && (fds[0].revents().unwrap_or(PollFlags::empty()) & PollFlags::POLLOUT)
                != PollFlags::empty())
    }

    pub fn completion_drain(&mut self) -> Result<u32, EngineError> {
        let descs = unsafe { (*self.completion_ring).consume_batch(4096) };
        let count = descs.len() as u32;

        for addr in &descs {
            self.umem.return_frame_addr(*addr);
        }

        Ok(count)
    }

    pub fn free_frame_count(&self) -> usize {
        self.umem.free_count()
    }

    pub fn fill_ring_available(&self) -> u32 {
        unsafe { (*self.fill_ring).available() }
    }

    pub fn rx_ring_available(&self) -> u32 {
        unsafe { (*self.rx_ring).available() }
    }
}

#[derive(Debug, Clone)]
pub struct RawPacket {
    pub data: Vec<u8>,
    pub addr: u64,
}
