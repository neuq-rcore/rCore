use fatfs::FileSystem;
use log::debug;

use crate::fat32::{Fat32FileSystem, Fat32IO};
use lazy_static::lazy_static;

use self::inode::Fat32Dir;
pub mod inode;

lazy_static! {
    pub static ref ROOT_FS: RootFs = {
        let fs = Fat32FileSystem::new(0);
        debug!("Filesystem initialized.");
        RootFs::new(fs)
    };
}

pub struct RootFs {
    inner: FileSystem<Fat32IO>,
}

unsafe impl Sync for RootFs {}
unsafe impl Send for RootFs {}

impl RootFs {
    pub fn new(raw_fs: FileSystem<Fat32IO>) -> Self {
        Self { inner: raw_fs }
    }

    pub fn root_dir(&'static self) -> Fat32Dir {
        Fat32Dir::from_root(self.inner.root_dir())
    }
}
