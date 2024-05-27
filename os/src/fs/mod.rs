use crate::{allocation::RefOrValue, fat32::{Fat32FileSystem, Fat32IO}};
use alloc::{string::String, sync::Arc, vec::Vec};
use fatfs::{Dir, File, FileSystem, LossyOemCpConverter, NullTimeProvider, Read};
use lazy_static::lazy_static;

pub type FatfsDir<'a> = Dir<'a, Fat32IO, NullTimeProvider, LossyOemCpConverter>;
pub type FatfsFile<'a> = File<'a, Fat32IO, NullTimeProvider, LossyOemCpConverter>;
pub type FatfsEntry<'a> = fatfs::DirEntry<'a, Fat32IO, NullTimeProvider, LossyOemCpConverter>;

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


pub struct Fat32File<'a> {
    inner: FatfsEntry<'a>,
}

pub struct Fat32Dir<'a> {
    inner: Fat32DirInner<'a>,
}

enum Fat32DirInner<'a> {
    Root(FatfsDir<'a>),
    Sub(FatfsEntry<'a>),
}

impl<'a> Fat32DirInner<'a> {
    pub fn from_root(root: FatfsDir<'a>) -> Self {
        Self::Root(root)
    }

    pub fn from_entry(entry: FatfsEntry<'a>) -> Self {
        Self::Sub(entry)
    }

    pub fn as_dir(&self) -> FatfsDir<'a> {
        match self {
            Self::Root(dir) => dir.clone(),
            Self::Sub(entry) => entry.to_dir(),
        }
    }
}

impl<'a> Fat32File<'a> {
    pub fn from_entry(entry: FatfsEntry<'a>) -> Self {
        Self { inner: entry }
    }

    pub fn len(&self) -> usize {
        self.inner.len() as usize
    }

    pub fn name(&self) -> Option<String> {
        Some(self.inner.file_name())
    }

    pub fn as_file(&self) -> FatfsFile<'a> {
        self.inner.to_file()
    }

    pub fn as_entry(&mut self) -> &mut FatfsEntry<'a> {
        &mut self.inner
    }
}

impl<'a> Fat32Dir<'a> {
    pub fn from_root(root: FatfsDir<'a>) -> Self {
        Self {
            inner: Fat32DirInner::from_root(root),
        }
    }

    pub fn from_entry(entry: FatfsEntry<'a>) -> Self {
        Self {
            inner: Fat32DirInner::from_entry(entry),
        }
    }

    pub fn as_dir(&self) -> FatfsDir<'a> {
        self.inner.as_dir()
    }

    pub fn as_entry(&mut self) -> Option<&mut FatfsEntry<'a>> {
        match &mut self.inner {
            Fat32DirInner::Root(_) => panic!("Root directory does not have an entry"),
            Fat32DirInner::Sub(entry) => Some(entry),
        }
    }

    pub fn name(&self) -> Option<String> {
        match self.inner {
            Fat32DirInner::Root(_) => None,
            Fat32DirInner::Sub(ref entry) => Some(entry.file_name()),
        }
    }

    // get a directory entry by name in the current directory
    fn match_dir(&self, name: &str) -> Option<Fat32Dir<'a>> {
        self.as_dir()
            .iter()
            .find(|entry| {
                entry
                    .as_ref()
                    .is_ok_and(|e| e.is_dir() && e.file_name() == name)
            })
            .map(|entry| Fat32Dir::from_entry(entry.unwrap()))
    }

    // get a file entry in the current directory
    fn match_file(&self, name: &str) -> Option<Fat32File<'a>> {
        self.as_dir()
            .iter()
            .find(|entry| {
                entry
                    .as_ref()
                    .is_ok_and(|e| e.is_file() && e.file_name() == name)
            })
            .map(|entry| Fat32File::from_entry(entry.unwrap()))
    }

    // This method doesn't check if the file was in the parent directory
    // but it ensure that the parent directory is valid
    // Also, if the parent directory is the root/current directory, it will return None as we can;t return a copy of the root/current directory
    pub fn get_parent_dir(&self, path: &str) -> Result<Fat32Dir<'a>, GetParentDirError> {
        let mut paths = path.split('/').into_iter();
        let first_path = paths.next();
        let mut next_path = paths.next();

        match first_path {
            None => Err(GetParentDirError::RootDir), // is root
            Some(first) => {
                let mut curr = first;
                if first == "." {
                    match next_path {
                        None => return Err(GetParentDirError::RootDir),
                        Some(p) => {
                            curr = p;
                            next_path = paths.next();
                        },
                    }
                }

                let mut cwd: RefOrValue<Fat32Dir> = RefOrValue::from_ref(self);

                while let Some(_) = next_path {
                    // next of next
                    next_path = paths.next();

                    match next_path {
                        None => return cwd.match_dir(curr).ok_or(GetParentDirError::InvalidPath),
                        Some(_) => match cwd.match_dir(curr) {
                            None => return Err(GetParentDirError::InvalidPath),
                            Some(dir) => {
                                cwd = RefOrValue::from_value(dir);
                                curr = next_path.unwrap();
                            },
                        },
                    }
                }

                Err(GetParentDirError::InvalidPath)
            }
        }
    }

    pub fn get_dir(&self, path: &str) -> Option<Self> {
        let mut paths = path.split('/').into_iter();
        let mut next_path = paths.next();
        let mut cwd = RefOrValue::from_ref(self);

        while let Some(curr) = next_path {
            next_path = paths.next();

            match next_path {
                // we've reach the last path, it should be a directory
                // In `get_file` it should be a file
                None => return cwd.match_dir(curr),
                // Just continue and into the next iteration
                Some(_) => match cwd.match_dir(curr) {
                    None => return None,
                    Some(dir) => cwd = RefOrValue::from_value(dir),
                },
            }
        }

        None
    }

    pub fn get_file(&self, path: &str) -> Option<Fat32File> {
        let path = if path.starts_with('/') {
            &path[1..]
        } else {
            path
        };

        let mut paths = path.split('/').into_iter();
        let mut next_path = paths.next();
        let mut cwd = RefOrValue::from_ref(self);

        while let Some(curr) = next_path {
            next_path = paths.next();

            match next_path {
                // we've reach the last path, it should be a file
                // In `get_dir` it should be a directory
                None => return cwd.match_file(curr),
                // Just continue and into the next iteration
                Some(_) => match cwd.match_dir(curr) {
                    None => return None,
                    Some(dir) => cwd = RefOrValue::from_value(dir),
                },
            }
        }

        None
    }

    pub fn read_file_as_buf(&self, path: &str) -> Option<Vec<u8>> {
        let file = self.get_file(path);

        match file {
            None => None, // fast path
            Some(file) => {
                let len = file.len();
                let mut buf: Vec<u8> = Vec::with_capacity(len);
                unsafe {
                    buf.set_len(len);
                }

                let slice = buf.as_mut();

                file.as_file().read_exact(slice).ok().map(|_| buf)
            }
        }
    }
}

pub enum GetParentDirError {
    RootDir,
    InvalidPath,
}
