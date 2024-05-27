#![no_main]
#![no_std]
#![feature(
    panic_info_message,
    slice_from_ptr_range,
    naked_functions,
    alloc_error_handler,
    vec_into_raw_parts,
)]

use core::{arch::asm, slice};

use log::{debug, info};
use sbi::shutdown;

use crate::fs::get_fs;

#[macro_use]
extern crate alloc;

#[macro_use]
mod stdio;
mod allocation;
mod boards;
mod config;
mod driver;
mod fat32;
mod fs;
mod lang_items;
mod logging;
mod mm;
mod sbi;
mod stack_trace;
mod sync;
mod syscall;
mod task;
mod timer;
mod trap;

// Since we've implemented filesystem, we will soon migrate to test suits from sdcard image
// global_asm!(include_str!("link_app.S"));

#[no_mangle]
fn main() {
    let test_cases = vec![
        "execve",
        "brk",
        "chdir",
        "clone",
        "close",
        "dup2",
        "dup",
        "exit",
        "fork",
        "fstat",
        "getcwd",
        "getdents",
        "getpid",
        "getppid",
        "gettimeofday",
        "mkdir_",
        "mmap",
        "mount",
        "munmap",
        "open",
        "openat",
        "pipe",
        "read",
        "sleep",
        "times",
        "umount",
        "uname",
        "unlink",
        "wait",
        "waitpid",
        "write",
        "yield",
    ];

    for name in test_cases.into_iter() {
        let buf = get_fs().root_dir().read_file_as_buf(name);

        match buf {
            Some(buf) => {
                task::kernel_create_process(&buf);

                info!("Running user apps '{}' from sdcard.img", name);
                task::run_tasks();
            }
            None => {
                info!("Test case '{}' not found. Skipping.", name);
            }
        }
    }

    debug!("All tests finished. Shutting down.")
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

    // heap initlization depends on logging
    mm::init();

    trap::init();
    trap::enable_timer_interrupt();
    timer::set_next_trigger();

    debug_env();

    main();

    shutdown(false);
}

fn debug_env() {
    use crate::sbi::console::UnionConsole;
    use sbi_spec::base::impl_id;

    info!("Hello, world!");

    debug!("SBI specification version: {0}", sbi_rt::get_spec_version());

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

    debug!("SBI implementation: {0}", sbi_impl);

    let console_type = match UnionConsole::instance() {
        UnionConsole::Legacy(_) => "Legacy",
        UnionConsole::Dbcn(_) => "DBCN",
    };

    debug!("Console type: {0}", console_type);
}

unsafe fn clear_bss() {
    extern "C" {
        fn sbss();
        fn ebss();
    }

    slice::from_mut_ptr_range(sbss as *mut u8..ebss as *mut u8).fill(0);
}
