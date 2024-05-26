use core::mem::forget;

use virtio_drivers::Hal;

use crate::mm::{
    frame::{frame_alloc_contiguous, frame_dealloc_contiguous},
    PhysAddr,
};
pub const VIRTIO0: usize = 0x1000_1000;

pub struct VirtioHal;

impl Hal for VirtioHal {
    fn dma_alloc(count: usize) -> virtio_drivers::PhysAddr {
        let pages = frame_alloc_contiguous(count).unwrap();

        let ppn_base = pages.last().unwrap().ppn;
        let begin_addr: PhysAddr = ppn_base.into();

        forget(pages);

        // debug!("[#####] dma_alloc: {:#x?}, count: {}", begin_addr, count);

        begin_addr.into()
    }

    fn dma_dealloc(paddr: virtio_drivers::PhysAddr, pages: usize) -> i32 {
        let ppn: PhysAddr = paddr.into();

        // debug!("[#####] dma_dealloc: {:#x?}, count: {}", paddr, pages);
        frame_dealloc_contiguous(ppn.into(), pages);
        0
    }
    fn phys_to_virt(paddr: virtio_drivers::PhysAddr) -> virtio_drivers::VirtAddr {
        // debug!("[#####] CONVERTING PHYS TO VIRT 0x{:016x}", paddr);
        paddr
    }

    fn virt_to_phys(vaddr: virtio_drivers::VirtAddr) -> virtio_drivers::PhysAddr {
        // let kernel_token = KERNEL_SPACE.exclusive_access().token();
        // let kernel_table = PageTable::from_token(kernel_token);

        // kernel_table.translate(vpn)
        // debug!("[#####] CONVERTING VIRT TO PHYS 0x{:016x}", vaddr);
        vaddr
    }
}
