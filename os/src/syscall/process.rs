use core::arch::asm;
use core::ffi::c_char;
use log::*;
use crate::boards::qemu::CLOCK_FREQ;
use crate::mm::page::PageTable;

use crate::task::processor::{current_task, current_user_token};
use crate::task::{exit_current_and_run_next, suspend_current_and_run_next, IDLE_PID};
use crate::timer::{TimeVal, get_timeval};

const SIGCHLD: usize = 17;

pub fn sys_exit(exit_code: i32) -> ! {
    info!("Application exited with code {}", exit_code);
    exit_current_and_run_next(exit_code);
    panic!("Unreachable in sys_exit!");
}

pub fn sys_yield() -> isize {
    suspend_current_and_run_next();
    0
}

pub fn sys_nanosleep(req: *mut TimeVal, _rem: *mut TimeVal) -> isize {
    match req.is_null() {
        true => -1,
        false => {
            let user_token = current_user_token();
            let len = core::mem::size_of::<TimeVal>();

            let mut req_time = TimeVal::zero();

            PageTable::copy_from_space(user_token, req as *const u8, &mut req_time as *mut TimeVal as *mut u8, len);

            debug!("Requested sleep, sec: {}, usec: {}", req_time.sec, req_time.usec);
            let loopcount = CLOCK_FREQ * req_time.sec as usize;

            for _ in 0..loopcount {
                unsafe {
                    asm!("nop");
                }
            }

            0
        }
    }
}

pub fn sys_get_time(ts: *mut TimeVal, _tz: i32) -> isize {
    match ts.is_null() {
        true => -1,
        false => {
            let user_token = current_user_token();
            let now = get_timeval();
            let len = core::mem::size_of::<TimeVal>();

            let copied = PageTable::copy_to_space(user_token, &now as *const _ as *const u8, ts as *mut u8, len);

            match copied == len {
                true => 0,
                false => -1,
            }
        }
    }
}

#[repr(C)]
struct Tms              
{                     
	tms_utime: i64,
	tms_stime: i64,
	tms_cutime: i64,
	tms_cstime: i64,
}

pub fn sys_times(tms: usize) -> isize {
    match tms == 0 {
        true => -1,
        false => {
            let user_token = current_user_token();
            let len = core::mem::size_of::<Tms>();
            let tm = Tms {
                tms_utime: 0,
                tms_stime: 0,
                tms_cutime: 0,
                tms_cstime: 0,
            };

            PageTable::copy_to_space(user_token, &tm as *const _ as *const u8, tms as *mut u8, len);

            0
        }
    }
}

fn sys_fork() -> isize {
    unimplemented!();
}

pub fn sys_clone(fn_ptr: usize, stack: usize, flags: usize) -> isize {
    if fn_ptr == SIGCHLD {
        return sys_fork();
    }

    unimplemented!();
}

pub fn sys_exec(pathname: *const u8, argv: *const *const c_char, envp: *const *const c_char) -> ! {
    //
    unreachable!();
}

pub fn sys_getppid() -> isize {
    let current_task = current_task();

    match current_task {
        // Should never happen, but we left it here for safety
        None => IDLE_PID as isize,
        Some(current_task) => {
            match current_task.exclusive_inner().parent {
                // we don't have a init process and did not implemented parent/child relationship
                None => 1,
                Some(ref p) => {
                    match p.upgrade() {
                        None => 1,
                        Some(p) => {
                            p.pid() as isize
                        }
                    }
                }
            }
        }
    }
}

pub fn sys_getpid() -> isize {
    current_task().unwrap().pid() as isize
}
