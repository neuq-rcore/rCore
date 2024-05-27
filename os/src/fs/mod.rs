use crate::fat32::{Fat32FileSystem, Fat32IO};
use alloc::sync::Arc;
use fatfs::FileSystem;
use lazy_static::lazy_static;

use self::inode::Fat32Dir;

pub mod inode;

lazy_static! {
    pub static ref ROOT_FS: Arc<RootFs> = Arc::new(RootFs::new(0));
}

pub fn get_fs() -> Arc<RootFs> {
    ROOT_FS.clone()
}

pub struct RootFs {
    pub fs: Arc<FileSystem<Fat32IO>>,
}

unsafe impl Sync for RootFs {}
unsafe impl Send for RootFs {}

impl RootFs {
    pub fn new(device_id: usize) -> Self {
        let fs = Arc::new(Fat32FileSystem::new(device_id));
        Self { fs }
    }

    pub fn root_dir(&self) -> Fat32Dir {
        Fat32Dir::from_root(self.fs.root_dir())
    }
}
