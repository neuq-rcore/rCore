use core::ptr::NonNull;

use alloc::boxed::Box;
use virtio_drivers::{device::blk::VirtIOBlk, transport::mmio::{MmioTransport, VirtIOHeader}};

use crate::{boards::get_board, driver::virt::VirtioHal, sync::UPSafeCell};

use super::{driver::IExt4Driver, Ext4Filesystem};

pub struct VirtDriver {
    virtio_blk: UPSafeCell<VirtIOBlk<VirtioHal, MmioTransport>>,
}

impl IExt4Driver for VirtDriver {
    fn read_blocks(&self, sector: usize, buf: &mut [u8]) {
        self.virtio_blk
            .exclusive_access()
            .read_blocks(sector, buf)
            .expect("Error occurred when reading VirtIOBlk");
    }

    fn write_blocks(&self, sector: usize, buf: &[u8]) {
        self.virtio_blk
            .exclusive_access()
            .write_blocks(sector, buf)
            .expect("Error occurred when writing VirtIOBlk");
    }
}

impl VirtDriver {
    pub fn create_fs(device_id: usize) -> Ext4Filesystem {
        let board = get_board();

        let pa = board.mmc_driver(device_id);

        // Kernel space is identity mapped
        let va = pa;

        let header = NonNull::new(va as *mut VirtIOHeader).unwrap();

        let transport =
            unsafe { MmioTransport::new(header).expect("Failed to create mmio transport") };

        let blk = VirtIOBlk::<VirtioHal, MmioTransport>::new(transport)
            .expect("Failed to create VirtIOBlk");

        let virtio_blk = UPSafeCell::new(blk);

        let driver = VirtDriver { virtio_blk };

        Ext4Filesystem::new(Box::new(driver))
    }
}
