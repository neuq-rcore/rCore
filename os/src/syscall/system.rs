use crate::mm::page::PageTable;

use crate::task::processor::current_user_token;
use core::ptr::copy_nonoverlapping;

#[repr(C)]
pub struct Utsname {
	sysname: [u8; 65],
	nodename: [u8; 65],
	release: [u8; 65],
	version: [u8; 65],
	machine: [u8; 65],
	domainname: [u8; 65]
}

const SYSNAME: &str = "neuqOS";
const NODENAME: &str = "neuqOS";
const RELEASE: &str = "0.1";
const VERSION: &str = "0.1";
const MACHINE: &str = "riscv64";
const DOMAINNAME: &str = "neuq.edu.cn";

impl Utsname {
    pub fn new() -> Self {
        let mut utsname = Utsname {
            sysname: [0; 65],
            nodename: [0; 65],
            release: [0; 65],
            version: [0; 65],
            machine: [0; 65],
            domainname: [0; 65],
        };

        unsafe {
            copy_nonoverlapping(SYSNAME.as_ptr(), utsname.sysname.as_mut_ptr(), SYSNAME.len());
            copy_nonoverlapping(NODENAME.as_ptr(), utsname.nodename.as_mut_ptr(), NODENAME.len());
            copy_nonoverlapping(RELEASE.as_ptr(), utsname.release.as_mut_ptr(), RELEASE.len());
            copy_nonoverlapping(VERSION.as_ptr(), utsname.version.as_mut_ptr(), VERSION.len());
            copy_nonoverlapping(MACHINE.as_ptr(), utsname.machine.as_mut_ptr(), MACHINE.len());
            copy_nonoverlapping(DOMAINNAME.as_ptr(), utsname.domainname.as_mut_ptr(), DOMAINNAME.len());
        }

        utsname
    }
}

pub fn sys_uname(uname: *mut Utsname) -> isize {
    match uname.is_null() {
        true => -1,
        false => {
            let un = Utsname::new();
            let len = core::mem::size_of::<Utsname>();
            let user_token = current_user_token();

            let copied  = PageTable::copy_to_space(user_token, &un as *const _ as *const u8, uname as *mut u8, len);

            copied as isize - len as isize
        }
    }
}