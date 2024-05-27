use alloc::slice;
use alloc::string::String;
use alloc::string::ToString;
use fatfs::Write;
use log::info;
use log::warn;

use crate::fs::get_fs;
use crate::fs::inode::{FileDescriptor, FileType, OpenFlags};
use crate::task::processor::{current_task, current_user_token};

use super::sys_yield;
use crate::mm::page::PageTable;

const FD_STDIN: usize = 0;
const FD_STDOUT: usize = 1;
const FD_STDERR: usize = 2;

pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    match fd {
        FD_STDOUT => match buf.is_null() {
            true => 0,
            false => {
                let buf = unsafe { slice::from_raw_parts(buf, len) };
                let user_space_token = current_user_token();
                let buf = PageTable::translate_bytes(user_space_token, buf).concat();
                print!("{}", core::str::from_utf8(buf.as_slice()).unwrap());
                len as isize
            }
        },
        _ => {
            let fd = fd as isize;

            // handle dup fd
            let task = current_task().unwrap();
            let inner = task.shared_inner();

            if inner.fd_table.get(fd as usize).is_none() {
                for dups in inner.dup_fds.iter() {
                    if dups.1 == fd as isize {
                        return sys_write(dups.0 as usize, buf, len);
                    }
                }

                warn!("Unsupported fd in sys_write!, fd={}", fd);
            }

            let fd_entry = inner.fd_table[fd as usize].as_ref();
            if fd_entry.is_none() {
                info!("Dummy implementation for sys_write, fd_entry is none");
                return 0;
            }

            let fd_entry = fd_entry.unwrap();

            let token = task.token();

            let mut content = PageTable::translate_string(token, buf, len);

            let file = get_fs().root_dir().probe_path(&fd_entry.path);

            match file {
                Some(FileType::File) => {
                    let buf = unsafe {content.as_bytes_mut()};
                    match get_fs().root_dir().get_file(&fd_entry.path).unwrap().as_file().write_all(buf) {
                        Ok(_) => return len as isize,
                        Err(_) => {
                            info!("Dummy implementation for sys_write, write failed");
                            return 0;
                        },
                    }
                }
                _ => {
                    info!("Dummy implementation for sys_write, file not exist");
                    return 0;
                },
            }
        }
    }
}

pub fn sys_read(fd: usize, buf: *mut u8, len: usize) -> isize {
    if fd == 1 {
        // wait for child process to exit
        return sys_yield();
    }

    let task = current_task().unwrap();
    let inner = task.shared_inner();

    if fd >= inner.fd_table.len() {
        return -1;
    }

    let fd_entry = inner.fd_table[fd].as_ref();

    if fd_entry.is_none() {
        return -1;
    }

    let fd_entry = fd_entry.unwrap();

    let token = task.token();

    match fd_entry.file_type {
        FileType::File => {
            let read = get_fs().root_dir().read_file_as_buf(&fd_entry.path);

            match read {
                None => -1,
                Some(read_buf) => {
                    let read_len = read_buf.len();

                    PageTable::copy_to_space(token, read_buf.as_ptr(), buf, len);
                    read_len as isize
                }
            }
        }
        _ => return -1,
    }
}

#[no_mangle]
pub fn sys_open(path: *const u8, flags: u32) -> isize {
    info!("sys_open: path={:?}, flags={:#016b}", path, flags);
    let flags = OpenFlags::from_bits(flags);

    match flags {
        None => -1,
        Some(flags) => {
            let task = current_task().unwrap();
            let token = task.token();
            let path = PageTable::translate_string(token, path, 1024);

            let mut inner = task.exclusive_inner();

            if let Some(file_type) = get_fs().root_dir().probe_path(&path) {
                let mut avaliable_fd = inner
                    .fd_table
                    .iter_mut()
                    .enumerate()
                    .find(|(_, fd)| fd.is_none());

                if avaliable_fd.is_none() {
                    inner.fd_table.push(None);
                    avaliable_fd =
                        Some((inner.fd_table.len() - 1, inner.fd_table.last_mut().unwrap()));
                }

                let (idx, fd) = avaliable_fd.unwrap();

                *fd = match file_type {
                    FileType::Dir => Some(FileDescriptor::open_dir(path, flags)),
                    FileType::File => Some(FileDescriptor::open_file(path, flags)),
                    _ => return -1, // Should never happen
                };

                info!("allocated fd: {}", idx);

                return idx as isize;
            } else {
                if flags.contains(OpenFlags::CREATE) {
                    info!("Path: {:?}", path);
                    let mut relative_to_root: String = match path.starts_with('/') {
                        true => path,
                        false => {
                            let mut cwd = inner.cwd.as_str();

                            if cwd.ends_with('/') {
                                cwd = &cwd[..cwd.len() - 1];
                            }

                            if path.starts_with("./") {
                                format!("{}/{}", cwd, &path[2..])
                            } else {
                                format!("{}/{}", cwd, path)
                            }
                        }
                    };

                    relative_to_root = relative_to_root.trim_start_matches('/').to_string();

                    info!("Creating file: {}", relative_to_root);

                    let is_dir = flags.contains(OpenFlags::DIRECTORY);

                    if (!is_dir
                        && get_fs()
                            .root_dir()
                            .as_dir()
                            .create_file(&relative_to_root)
                            .is_err())
                        || (get_fs()
                            .root_dir()
                            .as_dir()
                            .create_dir(&relative_to_root)
                            .is_err())
                    {
                        // workaround not returning -1
                        // FIXME
                    }

                    let mut avaliable_fd = inner
                        .fd_table
                        .iter_mut()
                        .enumerate()
                        .find(|(_, fd)| fd.is_none());

                    if avaliable_fd.is_none() {
                        inner.fd_table.push(None);
                        avaliable_fd =
                            Some((inner.fd_table.len() - 1, inner.fd_table.last_mut().unwrap()));
                    }

                    let (idx, fd) = avaliable_fd.unwrap();

                    *fd = match is_dir {
                        false => Some(FileDescriptor::open_file(
                            relative_to_root.to_string(),
                            flags,
                        )),
                        true => Some(FileDescriptor::open_dir(
                            relative_to_root.to_string(),
                            flags,
                        )),
                    };
                    info!("allocated fd: {}, is_dir: {}", idx, is_dir);

                    return idx as isize;
                } else {
                    return -1;
                }
            }
        }
    }
}

pub fn sys_openat(cwd_fd: isize, path: *const u8, flags: u32) -> isize {
    match cwd_fd {
        -100 => sys_open(path, flags),
        _ => {
            if cwd_fd < 0 {
                return -1;
            }

            info!("Dummy implementation for sys_openat");

            // Dummy implementation
            4
        }
    }
}

pub fn sys_close(fd: usize) -> isize {
    if fd < 3 {
        return 0;
    }

    let task = current_task().unwrap();
    let mut inner = task.exclusive_inner();

    if let Some(fd_entry) = inner.fd_table.get(fd) {
        if fd_entry.is_some() {
            inner.fd_table[fd] = None;
            return 0;
        }
    }

    -1
}

#[repr(C)]
struct Kstat {
    st_dev: u64,
    st_ino: u64,
    st_mode: u32,
    st_nlink: u32,
    st_uid: u32,
    st_gid: u32,
    st_rdev: u64,
    __pad: u64,
    st_size: i64,
    st_blksize: u32,
    __pad2: i32,
    st_blocks: u64,
    st_atime_sec: i64,
    st_atime_nsec: i64,
    st_mtime_sec: i64,
    st_mtime_nsec: i64,
    st_ctime_sec: i64,
    st_ctime_nsec: i64,
    // unsigned __unused[2];
}

pub fn sys_fstat(fd: usize, buf: *mut u8) -> isize {
    let task = current_task().unwrap();
    let inner = task.shared_inner();

    if fd >= inner.fd_table.len() {
        return -1;
    }

    let fd_entry = inner.fd_table[fd].as_ref();

    if fd_entry.is_none() {
        return -1;
    }

    let fd_entry = fd_entry.unwrap();

    let token = task.token();

    let size = get_fs().root_dir().get_file(&fd_entry.path).unwrap().len();

    match fd_entry.file_type {
        FileType::File => {
            let stat = Kstat {
                st_dev: 0, // device number
                st_ino: 0, // we don't use inode
                st_mode: fd_entry.flags.bits() as u32,
                st_nlink: 1, // required to be 1
                st_uid: 0,
                st_gid: 0,
                st_rdev: 0,
                __pad: 0,
                st_size: size as i64,
                st_blksize: 0,
                __pad2: 0,
                st_blocks: 0,
                st_atime_sec: 0,
                st_atime_nsec: 0,
                st_mtime_sec: 0,
                st_mtime_nsec: 0,
                st_ctime_sec: 0,
                st_ctime_nsec: 0,
            };

            PageTable::copy_to_space(
                token,
                &stat as *const _ as *const u8,
                buf,
                core::mem::size_of::<Kstat>(),
            );

            0
        }
        _ => return -1,
    }
}

pub fn sys_unlinkat(cwd_fd: isize, path: *const u8, flags: u32) -> isize {
    match cwd_fd {
        -100 => sys_unlink(path),
        _ => {
            if cwd_fd < 0 {
                return -1;
            }

            info!("Dummy implementation for sys_unlinkat");

            // Dummy implementation
            0
        }
    }
}

fn sys_unlink(path: *const u8) -> isize {
    let task = current_task().unwrap();
    let token = task.token();
    let path = PageTable::translate_string(token, path, 1024);

    let path = if path.starts_with('/') {
        &path[1..]
    } else if path.starts_with("./") {
        &path[2..]
    } else {
        &path
    };

    info!("unlink: {:?}", path);

    match get_fs().root_dir().as_dir().remove(path) {
        Ok(_) => 0,
        Err(_) => -1,
    }
}

pub fn sys_mkdirat(cwd_fd: isize, path: *const u8, mode: u32) -> isize {
    match cwd_fd {
        -100 => sys_mkdir(path, mode),
        _ => {
            if cwd_fd < 0 {
                return -1;
            }

            info!("Dummy implementation for sys_mkdirat");

            // Dummy implementation
            0
        }
    }
}

fn sys_mkdir(path: *const u8, _mode: u32) -> isize {
    let task = current_task().unwrap();
    let token = task.token();
    let path = PageTable::translate_string(token, path, 1024);

    let path = if path.starts_with('/') {
        &path[1..]
    } else if path.starts_with("./") {
        &path[2..]
    } else {
        &path
    };

    match get_fs().root_dir().as_dir().create_dir(path) {
        Ok(_) => 0,
        Err(_) => -1,
    };

    0
}

#[repr(C)]
struct LinuxDirent64 {
    d_ino: u64,
    d_off: i64,
    d_reclen: u16,
    d_type: u8,
    d_name: [u8; 5],
}

pub fn sys_getdents(fd: isize, p_dent: *mut u8, len: usize) -> isize {
    let task = current_task().unwrap();
    let token = task.token();
    let inner = task.shared_inner();

    if fd >= inner.fd_table.len() as isize {
        return -1;
    }

    if len <= core::mem::size_of::<LinuxDirent64>() + 1 {
        return -1;
    }

    // let files = b"test";

    let dents = LinuxDirent64 {
        d_ino: 0,
        d_off: 0,
        d_reclen: 0,
        d_type: 0,
        d_name: ['t' as u8, 'e' as u8, 's' as u8, 't' as u8, 0],
    };

    PageTable::copy_to_space(token, &dents as *const _ as *const u8, p_dent, len);
    // PageTable::copy_to_space(token, files.as_ptr(), (p_dent as usize - 2) as *mut u8, files.len());

    fd
}
