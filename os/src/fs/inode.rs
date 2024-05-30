use alloc::string::String;
use bitflags::bitflags;

bitflags! {
    #[derive(Clone, Copy)]
    pub struct OpenFlags: u32 {
        const RDONLY = 0;
        const WRONLY = 1 << 0;
        const RDWR = 1 << 1;
        const CREATE = 1 << 6;
        const TRUNC = 1 << 10;
        const DIRECTORY = 0x0200000;
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FileType {
    Undetermined,
    Dir,
    File,
    CharDevice, // unuse
}

#[derive(Clone)]
pub struct FileDescriptor {
    pub flags: OpenFlags,
    pub path: String,
    pub file_type: FileType,
}

impl FileDescriptor {
    pub fn open_dir(path: String, flags: OpenFlags) -> Self {
        Self {
            flags,
            path,
            file_type: FileType::Dir,
        }
    }

    pub fn open_file(path: String, flags: OpenFlags) -> Self {
        Self {
            flags,
            path,
            file_type: FileType::File,
        }
    }

    pub fn open_char_device(path: String, flags: OpenFlags) -> Self {
        Self {
            flags,
            path,
            file_type: FileType::CharDevice,
        }
    }

    pub fn open(path: String, flags: OpenFlags) -> Self {
        Self {
            flags,
            path,
            file_type: FileType::Undetermined,
        }
    }

    pub fn open_stdin() -> Self {
        Self::open_char_device(String::from("/dev/stdin"), OpenFlags::RDONLY)
    }

    pub fn open_stdout() -> Self {
        Self::open_char_device(String::from("/dev/stdout"), OpenFlags::WRONLY)
    }

    pub fn open_stderr() -> Self {
        Self::open_char_device(String::from("/dev/stderr"), OpenFlags::WRONLY)
    }

    pub fn path(&self) -> &str {
        &self.path
    }
}
