use std::ffi::CString;
use std::os::unix::io::RawFd;

use crate::rings::{COMPLETION_RING, FILL_RING, RX_RING, TX_RING};
use crate::umem::Umem;

#[derive(Debug, thiserror::Error)]
pub enum XskError {
    #[error("failed to create AF_XDP socket: {0}")]
    SocketCreate(String),
    #[error("failed to bind socket: {0}")]
    Bind(String),
    #[error("failed to setsockopt: {0}")]
    SetSockOpt(String),
    #[error("failed to mmap ring: {0}")]
    MmapRing(String),
    #[error("interface not found: {0}")]
    IfaceNotFound(String),
}

#[derive(Debug, Clone)]
pub struct XskConfig {
    pub rx_queue: u16,
    pub tx_queue: u16,
    pub xdp_flags: u32,
    pub bind_flags: u16,
    pub frame_count: u32,
}

impl Default for XskConfig {
    fn default() -> Self {
        Self {
            rx_queue: 0,
            tx_queue: 0,
            xdp_flags: 0,
            bind_flags: 0,
            frame_count: 4096,
        }
    }
}

const SOL_XDP: libc::c_int = 283;
const XDP_UMEM_REG: libc::c_int = 1;
const XDP_OPTIONS: libc::c_int = 4;
const XDP_MMAP_OFFSETS: libc::c_int = 3;
const XDP_USE_NEED_WAKEUP: u16 = 1 << 3;
const XDP_OPTIONS_ZEROCOPY: u32 = 1 << 1;

#[repr(C)]
struct xdp_umem_reg {
    addr: u64,
    len: u64,
    chunk_size: u32,
    headroom: u32,
    flags: u32,
}

#[repr(C)]
struct xdp_mmap_offsets {
    rx: xdp_ring_offset,
    tx: xdp_ring_offset,
    fill: xdp_ring_offset,
    completion: xdp_ring_offset,
}

#[repr(C)]
struct xdp_ring_offset {
    producer: u32,
    consumer: u32,
    desc: u32,
    flags: u32,
}

#[repr(C)]
struct sockaddr_xdp {
    sxdp_family: u16,
    sxdp_flags: u16,
    sxdp_ifindex: i32,
    sxdp_queue_id: u32,
    sxdp_shared_umem_fd: i32,
    sxdp_shared_umem_offset: u32,
    sxdp_shared_umem_size: u32,
}

pub struct XskSocket {
    fd: RawFd,
    ifindex: i32,
    fill_ring: *mut FILL_RING,
    completion_ring: *mut COMPLETION_RING,
    rx_ring: *mut RX_RING,
    tx_ring: *mut TX_RING,
    _umem_mapped: *mut libc::c_void,
    _umem_len: usize,
    _fill_mapped: *mut libc::c_void,
    _fill_len: usize,
    _completion_mapped: *mut libc::c_void,
    _completion_len: usize,
    _rx_mapped: *mut libc::c_void,
    _rx_len: usize,
    _tx_mapped: *mut libc::c_void,
    _tx_len: usize,
}

unsafe impl Send for XskSocket {}

impl XskSocket {
    pub fn new(ifname: &str, umem: &Umem, config: &XskConfig) -> Result<Self, XskError> {
        let ifname_c = CString::new(ifname).map_err(|e| XskError::IfaceNotFound(e.to_string()))?;
        let ifindex = unsafe { libc::if_nametoindex(ifname_c.as_ptr()) };
        if ifindex == 0 {
            return Err(XskError::IfaceNotFound(ifname.to_string()));
        }

        let fd = unsafe { libc::socket(libc::AF_XDP, libc::SOCK_RAW, 0) };
        if fd < 0 {
            return Err(XskError::SocketCreate(
                std::io::Error::last_os_error().to_string(),
            ));
        }

        let umem_reg = xdp_umem_reg {
            addr: umem.base_ptr() as u64,
            len: (umem.frame_count() * umem.frame_size()) as u64,
            chunk_size: umem.frame_size(),
            headroom: 256,
            flags: 0,
        };

        let ret = unsafe {
            libc::setsockopt(
                fd,
                SOL_XDP,
                XDP_UMEM_REG,
                &umem_reg as *const xdp_umem_reg as *const libc::c_void,
                std::mem::size_of::<xdp_umem_reg>() as libc::socklen_t,
            )
        };
        if ret != 0 {
            unsafe {
                libc::close(fd);
            }
            return Err(XskError::SetSockOpt(format!(
                "XDP_UMEM_REG: {}",
                std::io::Error::last_os_error()
            )));
        }

        let mut offsets = std::mem::MaybeUninit::<xdp_mmap_offsets>::uninit();
        let mut offsets_len = std::mem::size_of::<xdp_mmap_offsets>() as libc::socklen_t;

        let ret = unsafe {
            libc::getsockopt(
                fd,
                SOL_XDP,
                XDP_MMAP_OFFSETS,
                offsets.as_mut_ptr() as *mut libc::c_void,
                &mut offsets_len,
            )
        };
        if ret != 0 {
            unsafe {
                libc::close(fd);
            }
            return Err(XskError::SetSockOpt(format!(
                "XDP_MMAP_OFFSETS: {}",
                std::io::Error::last_os_error()
            )));
        }

        let offsets = unsafe { offsets.assume_init() };

        let fill_ring_size = (config.frame_count + 1) * std::mem::size_of::<u64>() as u32;
        let completion_ring_size = fill_ring_size;
        let rx_ring_size =
            (config.frame_count + 1) * std::mem::size_of::<crate::rings::xdp_desc>() as u32;
        let tx_ring_size = rx_ring_size;

        let fill_ring_meta = Self::mmap_ring(fd, offsets.fill, fill_ring_size)?;
        let completion_ring_meta = Self::mmap_ring(fd, offsets.completion, completion_ring_size)?;
        let rx_ring_meta = Self::mmap_ring(fd, offsets.rx, rx_ring_size)?;
        let tx_ring_meta = Self::mmap_ring(fd, offsets.tx, tx_ring_size)?;

        let fill_descs = Self::mmap_ring(
            fd,
            xdp_ring_offset {
                producer: 0,
                consumer: 0,
                desc: offsets.fill.desc,
                flags: 0,
            },
            fill_ring_size,
        )?;

        let completion_descs = Self::mmap_ring(
            fd,
            xdp_ring_offset {
                producer: 0,
                consumer: 0,
                desc: offsets.completion.desc,
                flags: 0,
            },
            completion_ring_size,
        )?;

        let rx_descs = Self::mmap_ring(
            fd,
            xdp_ring_offset {
                producer: 0,
                consumer: 0,
                desc: offsets.rx.desc,
                flags: 0,
            },
            rx_ring_size,
        )?;

        let tx_descs = Self::mmap_ring(
            fd,
            xdp_ring_offset {
                producer: 0,
                consumer: 0,
                desc: offsets.tx.desc,
                flags: 0,
            },
            tx_ring_size,
        )?;

        let fill_ring = unsafe {
            FILL_RING::new(
                fill_ring_meta as *mut crate::rings::xdp_umem_ring,
                fill_descs as *mut u64,
                config.frame_count,
            )
        };

        let completion_ring = unsafe {
            COMPLETION_RING::new(
                completion_ring_meta as *mut crate::rings::xdp_umem_ring,
                completion_descs as *mut u64,
                config.frame_count,
            )
        };

        let rx_ring = unsafe {
            RX_RING::new(
                rx_ring_meta as *mut crate::rings::xdp_ring,
                rx_descs as *mut crate::rings::xdp_desc,
                config.frame_count,
            )
        };

        let tx_ring = unsafe {
            TX_RING::new(
                tx_ring_meta as *mut crate::rings::xdp_ring,
                tx_descs as *mut crate::rings::xdp_desc,
                config.frame_count,
            )
        };

        let mut sxdp = sockaddr_xdp {
            sxdp_family: libc::AF_XDP as u16,
            sxdp_flags: config.xdp_flags as u16 | XDP_USE_NEED_WAKEUP,
            sxdp_ifindex: ifindex,
            sxdp_queue_id: config.rx_queue as u32,
            sxdp_shared_umem_fd: -1,
            sxdp_shared_umem_offset: 0,
            sxdp_shared_umem_size: 0,
        };

        let ret = unsafe {
            libc::bind(
                fd,
                &sxdp as *const sockaddr_xdp as *const libc::sockaddr,
                std::mem::size_of::<sockaddr_xdp>() as libc::socklen_t,
            )
        };
        if ret != 0 {
            unsafe {
                libc::close(fd);
            }
            return Err(XskError::Bind(std::io::Error::last_os_error().to_string()));
        }

        Ok(Self {
            fd,
            ifindex,
            fill_ring,
            completion_ring,
            rx_ring,
            tx_ring,
            _umem_mapped: std::ptr::null_mut(),
            _umem_len: 0,
            _fill_mapped: fill_ring_meta,
            _fill_len: fill_ring_size as usize,
            _completion_mapped: completion_ring_meta,
            _completion_len: completion_ring_size as usize,
            _rx_mapped: rx_ring_meta,
            _rx_len: rx_ring_size as usize,
            _tx_mapped: tx_ring_meta,
            _tx_len: tx_ring_size as usize,
        })
    }

    fn mmap_ring(
        fd: RawFd,
        offset: xdp_ring_offset,
        size: u32,
    ) -> Result<*mut libc::c_void, XskError> {
        let ptr = unsafe {
            libc::mmap(
                std::ptr::null_mut(),
                size as usize,
                libc::PROT_READ | libc::PROT_WRITE,
                libc::MAP_SHARED,
                fd,
                (offset.producer) as libc::off_t,
            )
        };
        if ptr == libc::MAP_FAILED {
            return Err(XskError::MmapRing(
                std::io::Error::last_os_error().to_string(),
            ));
        }
        Ok(ptr)
    }

    pub fn fill_ring(&self) -> *mut FILL_RING {
        self.fill_ring
    }

    pub fn completion_ring(&self) -> *mut COMPLETION_RING {
        self.completion_ring
    }

    pub fn rx_ring(&self) -> *mut RX_RING {
        self.rx_ring
    }

    pub fn tx_ring(&self) -> *mut TX_RING {
        self.tx_ring
    }

    pub fn rx_fd(&self) -> RawFd {
        self.fd
    }

    pub fn tx_fd(&self) -> RawFd {
        self.fd
    }

    pub fn fd(&self) -> RawFd {
        self.fd
    }

    pub fn ifindex(&self) -> i32 {
        self.ifindex
    }
}

impl Drop for XskSocket {
    fn drop(&mut self) {
        unsafe {
            libc::close(self.fd);

            if !self._fill_mapped.is_null() {
                libc::munmap(self._fill_mapped, self._fill_len);
            }
            if !self._completion_mapped.is_null() {
                libc::munmap(self._completion_mapped, self._completion_len);
            }
            if !self._rx_mapped.is_null() {
                libc::munmap(self._rx_mapped, self._rx_len);
            }
            if !self._tx_mapped.is_null() {
                libc::munmap(self._tx_mapped, self._tx_len);
            }
        }
    }
}
