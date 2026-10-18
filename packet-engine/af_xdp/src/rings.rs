#![allow(non_camel_case_types)]
#![allow(dead_code)]

use std::ptr;

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct xdp_desc {
    pub addr: u64,
    pub len: u32,
    pub options: u32,
}

const PROD_MASK: u32 = 0xFFFF;
const PROD_IDX_MASK: u32 = 0xFFFFFFFF;

#[repr(C)]
pub struct xdp_ring {
    pub producer: u32,
    pub consumer: u32,
    pub mask: u32,
    pub offset: u32,
}

#[repr(C)]
pub struct xdp_umem_ring {
    pub producer: u32,
    pub consumer: u32,
    pub mask: u32,
    pub offset: u32,
}

macro_rules! impl_ring {
    ($name:ident, $ring_type:ty, $entry:ty) => {
        pub struct $name {
            ring: *mut $ring_type,
            desc_ptr: *mut $entry,
            size: u32,
            capacity: u32,
        }

        unsafe impl Send for $name {}
        unsafe impl Sync for $name {}

        impl $name {
            pub fn new(ring: *mut $ring_type, desc_ptr: *mut $entry, size: u32) -> Self {
                Self {
                    ring,
                    desc_ptr,
                    size,
                    capacity: size / std::mem::size_of::<$entry>() as u32 - 1,
                }
            }

            pub fn available(&self) -> u32 {
                unsafe {
                    let r = &*self.ring;
                    self.capacity - (r.producer.wrapping_sub(r.consumer) & self.capacity)
                }
            }

            pub fn produce(&self, frames: &[$entry]) -> u32 {
                unsafe {
                    let r = &mut *self.ring;
                    let mut prod = r.producer;
                    let mut count = 0u32;

                    for frame in frames {
                        if (prod.wrapping_sub(r.consumer)) & PROD_IDX_MASK > self.capacity {
                            break;
                        }
                        let idx = (prod & self.capacity) as usize;
                        ptr::write_volatile(self.desc_ptr.add(idx), *frame);
                        prod = prod.wrapping_add(1);
                        count += 1;
                    }

                    ptr::write_volatile(&mut r.producer, prod);
                    count
                }
            }

            pub fn consume_batch(&self, max: u32) -> Vec<$entry> {
                unsafe {
                    let r = &*self.ring;
                    let cons = r.consumer;
                    let avail = (r.producer.wrapping_sub(cons)) & self.capacity;
                    let to_consume = max.min(avail);

                    let mut descs = Vec::with_capacity(to_consume as usize);
                    for i in 0..to_consume {
                        let idx = ((cons + i) & self.capacity) as usize;
                        descs.push(ptr::read_volatile(self.desc_ptr.add(idx)));
                    }

                    let new_cons = cons.wrapping_add(to_consume);
                    let r_mut = &mut *self.ring;
                    ptr::write_volatile(&mut r_mut.consumer, new_cons);
                    descs
                }
            }
        }
    };
}

impl_ring!(FILL_RING, xdp_umem_ring, u64);
impl_ring!(COMPLETION_RING, xdp_umem_ring, u64);
impl_ring!(TX_RING, xdp_ring, xdp_desc);
impl_ring!(RX_RING, xdp_ring, xdp_desc);

pub fn ring_size(frame_count: u32) -> u32 {
    (frame_count + 1) * std::mem::size_of::<u64>() as u32
}

pub fn desc_ring_size(frame_count: u32) -> u32 {
    (frame_count + 1) * std::mem::size_of::<xdp_desc>() as u32
}
