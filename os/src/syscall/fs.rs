use alloc::slice;

use crate::task::processor::{current_task, current_user_token};

use crate::mm::page::PageTable;

const FD_STDIN: usize = 0;
const FD_STDOUT: usize = 1;
const FD_STDERR: usize = 2;

pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    match fd {
        FD_STDOUT => {
            let buf = unsafe { slice::from_raw_parts(buf, len) };
            let user_space_token = current_user_token();
            let buf = PageTable::translate_bytes(user_space_token, buf).concat();
            print!("{}", core::str::from_utf8(buf.as_slice()).unwrap());
            len as isize
        }
        _ => {
            let task = current_task().unwrap();
            let inner = task.shared_inner();

            for dups in inner.dup_fds.iter() {
                if dups.1 == fd as isize {
                    return sys_write(dups.0 as usize, buf, len);
                }
            }

            panic!("Unsupported fd in sys_write!, fd={}", fd);
        }
    }
}
