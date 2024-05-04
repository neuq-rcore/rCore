#![no_main]
#![no_std]
#![feature(panic_info_message)]

use core::arch::global_asm;

#[macro_use]
mod console;
pub mod batch;
mod lang_items;
mod sbi;
mod sync;
pub mod syscall;
pub mod trap;

global_asm!(include_str!("entry.asm"));
global_asm!(include_str!("link_app.S"));

fn clear_bss() {
    extern "C" {
        fn sbss();
        fn ebss();
    }
    // (sbss as usize..ebss as usize).for_each(|addr| {
    //     unsafe { (addr as *mut u8).write_volatile(0) }
    // });
    unsafe {
        core::slice::from_raw_parts_mut(sbss as usize as *mut u8, ebss as usize - sbss as usize).fill(0);
    };
}

#[no_mangle]
fn rust_main() -> ! {
    clear_bss();
    trap::init();
    batch::init();
    batch::run_next_app();
}
