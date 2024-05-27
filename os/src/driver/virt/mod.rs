use core::mem::forget;
use log::error;

use virtio_drivers::Hal;

use crate::mm::{
    frame::{frame_alloc_contiguous, frame_dealloc_contiguous},
    PhysAddr, VirtAddr
};

use crate::mm::KERNEL_SPACE;

use core::ptr::NonNull;
pub const VIRTIO0: usize = 0x1000_1000;

pub struct VirtioHal;

unsafe impl Hal for VirtioHal {
    fn dma_alloc(
        pages: usize,
        _direction: virtio_drivers::BufferDirection,
    ) -> (virtio_drivers::PhysAddr, NonNull<u8>) {
        let pages = frame_alloc_contiguous(pages).unwrap();

        let ppn_base = pages.last().unwrap().ppn;
        let begin_addr: PhysAddr = ppn_base.into();

        forget(pages);

        let pa = begin_addr.into();
        let va = NonNull::new((pa) as *mut u8).unwrap();

        (pa, va)
    }

    unsafe fn dma_dealloc(
        paddr: virtio_drivers::PhysAddr,
        _vaddr: core::ptr::NonNull<u8>,
        pages: usize,
    ) -> i32 {
        let ppn: PhysAddr = paddr.into();

        frame_dealloc_contiguous(ppn.into(), pages);
        0
    }

    unsafe fn mmio_phys_to_virt(
        paddr: virtio_drivers::PhysAddr,
        _size: usize,
    ) -> core::ptr::NonNull<u8> {
        error!("mmio_pa2va: {:#08x}", paddr);
        // we use identity mapping
        NonNull::new((usize::from(paddr)) as *mut u8).unwrap()
    }

    unsafe fn share(
        buffer: core::ptr::NonNull<[u8]>,
        _direction: virtio_drivers::BufferDirection,
    ) -> virtio_drivers::PhysAddr {
        let va = buffer.as_ptr() as *mut u8 as usize;
        let pa = KERNEL_SPACE.exclusive_access().table().translate_va(VirtAddr::from(va)).unwrap();
        error!("mmio_va2pa: {:#08x} -> {:#08x}", va, pa);
        pa.into()
    }

    unsafe fn unshare(
        _paddr: virtio_drivers::PhysAddr,
        _buffer: core::ptr::NonNull<[u8]>,
        _direction: virtio_drivers::BufferDirection,
    ) {
        // Do nothing
    }
}
