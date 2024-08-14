use ext4_rs::Ext4;
use alloc::sync::Arc;
use crate::ext4::virt::VirtDriver;

mod inode;
mod path;

pub struct Ext4RootFs {
    pub fs: Ext4,
}

impl Ext4RootFs {
    pub fn new(device_id: usize) -> Self {
        let driver = VirtDriver::create_fs(device_id);
        Self {
            fs: Ext4::open(Arc::new(driver)),
        }
    }

    pub fn fs(&self) -> &Ext4 {
        &self.fs
    }
}