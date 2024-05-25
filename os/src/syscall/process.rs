use crate::boards::qemu::CLOCK_FREQ;
use crate::mm::page::PageTable;
use crate::task::TaskManager::add_to_waiting;
use crate::trap::{disable_timer_interrupt, enable_timer_interrupt};
use alloc::sync::Arc;
use core::arch::asm;
use core::ffi::c_char;
use log::*;

use crate::task::processor::{current_task, current_user_token};
use crate::task::{exit_current_and_run_next, suspend_current_and_run_next, IDLE_PID};
use crate::timer::{get_timeval, TimeVal};

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

            PageTable::copy_from_space(
                user_token,
                req as *const u8,
                &mut req_time as *mut TimeVal as *mut u8,
                len,
            );

            debug!(
                "Requested sleep, sec: {}, usec: {}",
                req_time.sec, req_time.usec
            );
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

            let copied = PageTable::copy_to_space(
                user_token,
                &now as *const _ as *const u8,
                ts as *mut u8,
                len,
            );

            match copied == len {
                true => 0,
                false => -1,
            }
        }
    }
}

#[repr(C)]
struct Tms {
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

            PageTable::copy_to_space(
                user_token,
                &tm as *const _ as *const u8,
                tms as *mut u8,
                len,
            );

            0
        }
    }
}

fn sys_fork() -> isize {
    use crate::task::TaskManager::add_task;

    let current_task = current_task().unwrap();
    let child_task = current_task.fork();
    let child_pid = child_task.pid();

    let child_trap_cx = child_task.exclusive_inner().trap_ctx();
    child_trap_cx.x[10] = 0; // child return value

    add_task(child_task);
    child_pid as isize
}

pub fn sys_clone(flags: usize, sp: usize, ptid: usize) -> isize {
    info!("[sys_clone] arg0: {}, arg1: {}, arg2: {}", flags, sp, ptid);

    if flags == SIGCHLD && sp == 0 {
        return sys_fork();
    }

    use crate::task::TaskManager::add_task;
    let current_task = current_task().unwrap();
    let child_task = current_task.fork();
    let child_pid = child_task.pid();

    let child_trap_cx = child_task.exclusive_inner().trap_ctx();
    child_trap_cx.x[10] = 0; // child return value
    child_trap_cx.x[2] = sp; // sp

    add_task(child_task);
    child_pid as isize
}

pub fn sys_exec(pathname: *const u8, argv: *const *const c_char, envp: *const *const c_char) -> ! {
    let token = current_user_token();

    // TODO: read the file and load the elf

    unimplemented!()
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
                Some(ref p) => match p.upgrade() {
                    None => 1,
                    Some(p) => p.pid() as isize,
                },
            }
        }
    }
}

pub fn sys_getpid() -> isize {
    current_task().unwrap().pid() as isize
}

fn sys_waitpid_inner(pid: isize, code: *mut isize) -> isize {
    let task = current_task().unwrap();
    let token = task.token();

    loop {
        let mut inner = task.exclusive_inner();
        // find a child process
        if !inner
            .children
            .iter()
            .any(|p| pid == -1 || pid as usize == p.pid())
        {
            return -1;
        }

        let pair = inner
            .children
            .iter()
            .enumerate()
            .find(|(_, p)| p.is_zombie() && (pid == -1 || pid as usize == p.pid()));

        if let Some((idx, _)) = pair {
            let child = inner.children.remove(idx);
            assert_eq!(Arc::strong_count(&child), 1);
            let found_pid = child.pid();
            let exit_code = child.shared_inner().exit_code;
            info!(
                "Found child process: {}, exit code: {}, I am {}",
                found_pid, exit_code, task.pid()
            );

            let exit_code = (exit_code << 8) & 0xff00;

            if !code.is_null() {
                PageTable::copy_to_space(
                    token,
                    &exit_code as *const i32 as *const u8,
                    code as *mut u8,
                    core::mem::size_of::<i32>(),
                );
                info!("Copied exit code to user space");
            }

            return found_pid as isize;
        } else {
            // Wait until the child process exits
            info!("No child process found, waiting...");
            drop(inner);
            let task = task.clone();
            let assertion = Arc::new(move || {
                let task = task.clone();
                let inner = task.shared_inner();
                inner
                    .children
                    .iter()
                    .any(|p| (pid == -1 || p.pid() == pid as usize) && p.is_zombie())
            });
            add_to_waiting(current_task().unwrap(), assertion);
            suspend_current_and_run_next();
        }
    }
}

pub fn sys_waitpid(pid: isize, code: *mut isize, _options: usize) -> isize {
    sys_waitpid_inner(pid, code)
}
