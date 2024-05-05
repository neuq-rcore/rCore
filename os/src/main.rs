#![no_main]
#![no_std]
#![feature(panic_info_message)]

use core::arch::global_asm;

use sbi::shutdown;

#[macro_use]
pub mod stdio;
pub mod batch;
mod lang_items;
mod sbi;
mod stack_trace;
mod sync;
pub mod syscall;
pub mod trap;

global_asm!(include_str!("entry.asm"));
global_asm!(include_str!("link_app.S"));

#[no_mangle]
fn main() {
    batch::run_next_app();
}

#[no_mangle]
unsafe extern "C" fn __kernel_start_main() -> ! {
    clear_bss();

    trap::init();
    batch::init();

    main();

    shutdown(false);
}

unsafe fn clear_bss() {
    extern "C" {
        fn sbss();
        fn ebss();
    }

    core::slice::from_raw_parts_mut(sbss as usize as *mut u8, ebss as usize - sbss as usize)
        .fill(0);
}
