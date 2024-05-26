use alloc::{string::String, vec::Vec};
use bitflags::bitflags;
use fatfs::{Dir, File, LossyOemCpConverter, NullTimeProvider, Read};

use crate::{allocation::RefOrValue, fat32::Fat32IO};

pub type FatfsDir<'a> = Dir<'a, Fat32IO, NullTimeProvider, LossyOemCpConverter>;
pub type FatfsFile<'a> = File<'a, Fat32IO, NullTimeProvider, LossyOemCpConverter>;
pub type FatfsEntry<'a> = fatfs::DirEntry<'a, Fat32IO, NullTimeProvider, LossyOemCpConverter>;

bitflags! {
    pub struct OpenFlags: u32 {
        const RDONLY = 0;
        const WRONLY = 1 << 0;
        const RDWR = 1 << 1;
        const CREATE = 1 << 9;
        const TRUNC = 1 << 10;
    }
}

impl OpenFlags {
    /// Do not check validity for simplicity
    /// Return (readable, writable)
    pub fn read_write(&self) -> (bool, bool) {
        if self.is_empty() {
            (true, false)
        } else if self.contains(Self::WRONLY) {
            (false, true)
        } else {
            (true, true)
        }
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

    pub fn inner(&self) -> FatfsFile<'a> {
        self.inner.to_file()
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

    pub fn inner(&self) -> FatfsDir<'a> {
        self.inner.as_dir()
    }

    pub fn name(&self) -> Option<String> {
        match self.inner {
            Fat32DirInner::Root(_) => None,
            Fat32DirInner::Sub(ref entry) => Some(entry.file_name()),
        }
    }

    // get a directory entry by name in the current directory
    fn match_dir(&self, name: &str) -> Option<Fat32Dir<'a>> {
        self.inner()
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
        self.inner()
            .iter()
            .find(|entry| {
                entry
                    .as_ref()
                    .is_ok_and(|e| e.is_file() && e.file_name() == name)
            })
            .map(|entry| Fat32File::from_entry(entry.unwrap()))
    }

    pub fn get_dir(&self, path: &str) -> Option<Self> {
        let mut paths = path.split('/').into_iter();
        let mut next_path = paths.next();
        let mut cwd: RefOrValue<Fat32Dir> = RefOrValue::from_ref(self);

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
        let mut paths = path.split('/').into_iter();
        let mut next_path = paths.next();
        let mut cwd: RefOrValue<Fat32Dir> = RefOrValue::from_ref(self);

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
        self.get_file(path).map(|file| {
            let len = file.len();
            let mut buf: Vec<u8> = Vec::with_capacity(len);
            unsafe {
                buf.set_len(len);
            }

            let slice = buf.as_mut();

            file.inner().read_exact(slice).unwrap();
            buf
        })
    }
}
