use alloc::{boxed::Box, vec::Vec};
use driver::IExt4Driver;
use ext4_rs::BlockDevice;

pub mod driver;

pub mod virt;
pub mod vf2;

pub struct Ext4Filesystem {
    driver: Box<dyn IExt4Driver>,
}

impl Ext4Filesystem {
    pub fn new(driver: Box<dyn IExt4Driver>) -> Self {
        Self { driver }
    }
}

unsafe impl Send for Ext4Filesystem {}
unsafe impl Sync for Ext4Filesystem {}

impl BlockDevice for Ext4Filesystem {
    fn read_offset(&self, offset: usize) -> Vec<u8> {
        let mut buf = vec![0u8; 512];

        self.driver.read_blocks(offset, &mut buf);
        buf
    }

    fn write_offset(&self, offset: usize, data: &[u8]) {
        self.driver.write_blocks(offset, data);
    }
}
