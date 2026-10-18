use std::alloc::{alloc_zeroed, Layout};
use std::cell::RefCell;

use crate::{FRAME_SIZE, HEADROOM};

const NUM_FRAMES: usize = 4096;

#[derive(Debug, thiserror::Error)]
pub enum UmemError {
    #[error("failed to allocate UMEM: mmap failed")]
    MmapFailed,
    #[error("UMEM full: no free frames")]
    Full,
    #[error("invalid frame address: {0}")]
    InvalidFrame(u64),
}

#[derive(Debug, Clone)]
pub struct UmemConfig {
    pub frame_size: u32,
    pub frame_count: u32,
    pub headroom: u32,
    pub metadata_len: u32,
}

impl Default for UmemConfig {
    fn default() -> Self {
        Self {
            frame_size: FRAME_SIZE as u32,
            frame_count: NUM_FRAMES as u32,
            headroom: HEADROOM as u32,
            metadata_len: 0,
        }
    }
}

pub struct Umem {
    base: *mut u8,
    frame_count: u32,
    frame_size: u32,
    headroom: u32,
    free_stack: RefCell<Vec<u64>>,
    total_size: usize,
}

unsafe impl Send for Umem {}

impl Umem {
    pub fn new(config: &UmemConfig) -> Result<Self, UmemError> {
        let total_size = config.frame_count as usize * config.frame_size as usize;

        let layout =
            Layout::from_size_align(total_size, 4096).map_err(|_| UmemError::MmapFailed)?;

        let base = unsafe { alloc_zeroed(layout) };
        if base.is_null() {
            return Err(UmemError::MmapFailed);
        }

        let mut free_stack = Vec::with_capacity(config.frame_count as usize);
        for i in 0..config.frame_count {
            let addr = (i as u64) * (config.frame_size as u64);
            free_stack.push(addr);
        }

        Ok(Self {
            base,
            frame_count: config.frame_count,
            frame_size: config.frame_size,
            headroom: config.headroom,
            free_stack: RefCell::new(free_stack),
            total_size,
        })
    }

    pub fn next_free_frame_addr(&self) -> Result<u64, UmemError> {
        self.free_stack.borrow_mut().pop().ok_or(UmemError::Full)
    }

    pub fn return_frame_addr(&self, addr: u64) {
        self.free_stack.borrow_mut().push(addr);
    }

    pub fn frame_base_ptr(&self, frame_addr: u64) -> *mut u8 {
        unsafe { self.base.add(frame_addr as usize) }
    }

    pub fn frame_data_ptr(&self, frame_addr: u64) -> *mut u8 {
        unsafe { self.frame_base_ptr(frame_addr).add(self.headroom as usize) }
    }

    pub fn frame_count(&self) -> u32 {
        self.frame_count
    }

    pub fn frame_size(&self) -> u32 {
        self.frame_size
    }

    pub fn base_ptr(&self) -> *const u8 {
        self.base
    }

    pub fn base_ptr_mut(&mut self) -> *mut u8 {
        self.base
    }

    pub fn free_count(&self) -> usize {
        self.free_stack.borrow().len()
    }
}

impl Drop for Umem {
    fn drop(&mut self) {
        unsafe {
            std::alloc::dealloc(
                self.base,
                Layout::from_size_align(self.total_size, 4096).unwrap(),
            );
        }
    }
}
