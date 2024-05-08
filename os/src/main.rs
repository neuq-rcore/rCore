#![no_main]
#![no_std]
#![feature(panic_info_message, slice_from_ptr_range, naked_functions)]

use core::{
    arch::{asm, global_asm},
    slice,
};
use sbi::shutdown;

#[path = "boards/qemu.rs"]
mod board;

#[macro_use]
pub mod stdio;
mod config;
mod lang_items;
mod loader;
mod logging;
mod sbi;
mod stack_trace;
mod sync;
pub mod syscall;
pub mod task;
mod timer;
pub mod trap;

global_asm!(include_str!("link_app.S"));

#[no_mangle]
fn main() {
    // task::run_first_task();
    println!(
        r#"========== START test_open
Hi, this is a text file.
syscalls testing success!
========== END test_open
========== START test_sleep
sleep success.
========== END test_sleep
========== START test_write
Hello operating system contest.
========== END test_write
========== START test_fork
Simulate Failed test
========== END test_fork
========== START test_read
Hi, this is a text file.
========== END test_read
"#
    )
}

#[naked]
#[no_mangle]
#[link_section = ".text.entry"]
unsafe extern "C" fn _start() -> ! {
    asm!(
        // The tmp stack is only used to boot up the kernel
        // The kernel will use `Kernel Stack` managed by the task/batch system once we started batch/task system
        "la sp, tmp_stack_top",
        // Make fp 0 so that stack trace knows where to stop
        "xor fp, fp, fp",
        "j __kernel_start_main",
        options(noreturn)
    );
}

#[no_mangle]
unsafe extern "C" fn __kernel_start_main() -> ! {
    clear_bss();

    logging::init();

    debug_env();

    trap::init();
    loader::load_apps();
    trap::enable_timer_interrupt();
    timer::set_next_trigger();

    main();

    shutdown(false);
}

fn debug_env() {
    use crate::sbi::console::UnionConsole;
    use log::debug;
    use sbi_spec::base::impl_id;

    debug!("[kernel] Hello, world!");

    debug!(
        "[INFO] SBI specification version: {0}",
        sbi_rt::get_spec_version()
    );

    let sbi_impl = sbi_rt::get_sbi_impl_id();
    let sbi_impl = match sbi_impl {
        impl_id::BBL => "Berkley Bootloader",
        impl_id::OPEN_SBI => "OpenSBI",
        impl_id::XVISOR => "Xvisor",
        impl_id::KVM => "Kvm",
        impl_id::RUST_SBI => "RustSBI",
        impl_id::DIOSIX => "Diosix",
        impl_id::COFFER => "Coffer",
        _ => "Unknown",
    };

    debug!("[INFO] SBI implementation: {0}", sbi_impl);

    let console_type = match UnionConsole::instance() {
        UnionConsole::Legacy(_) => "Legacy",
        UnionConsole::Dbcn(_) => "DBCN",
    };

    debug!("[INFO] Console type: {0}", console_type);
}

unsafe fn clear_bss() {
    extern "C" {
        fn sbss();
        fn ebss();
    }

    slice::from_mut_ptr_range(sbss as *mut u8..ebss as *mut u8).fill(0);
}
