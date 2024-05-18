use log::info;
use riscv::paging::PageTable;

use crate::task::{current_user_token, exit_current_and_run_next, suspend_current_and_run_next};
use crate::timer::{TimeVal, get_timeval};

pub fn sys_exit(exit_code: i32) -> ! {
    info!("Application exited with code {}", exit_code);
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

pub fn sys_yield() -> isize {
    suspend_current_and_run_next();
    0
}

pub fn sys_get_time(ts: *mut TimeVal, _tz: i32) -> isize {
    match ts.is_null() {
        true => -1,
        false => {
            let now = get_timeval();
            unimplemented!();

            // We can not directly make `*ts = now` as `ts` is in userspace

            // TODO: Implement a general mem copy function to copy continuous memory to userspace
            // We should also handle cases where the data is across page boundaries

            // Proposal
            // 1. calculate the length of the data to copy
            // let len = size_of::<TimeVal>();

            // 2. Copy the data to the userspace
            // PageTable::copy_to_space(space_token, src, dst, len);

            // In our case:
            // PageTable::copy_to_space(space_token, &now as *const u8, ts as *mut u8, len)
            0
        }
    }
}
