use alloc::slice;

use crate::{mm::page::PageTable, task::current_user_token};

const FD_STDOUT: usize = 1;

pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    match fd {
        FD_STDOUT => {
            let buf = unsafe { slice::from_raw_parts(buf, len) };
            let user_space_token = current_user_token();
            let buf = PageTable::translate_bytes(user_space_token, buf);
            for b in buf {
                print!("{}", core::str::from_utf8(b).unwrap());
            }
            len as isize
        }
        _ => {
            panic!("Unsupported fd in sys_write!");
        }
    }
}
