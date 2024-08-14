#![no_main]
#![no_std]
#![feature(
    panic_info_message,
    slice_from_ptr_range,
    naked_functions,
    alloc_error_handler,
    vec_into_raw_parts
)]
#![allow(clippy::uninit_vec)]
#![allow(unused)]
// When we have multiple if-else branches, all branches must be `if let Some()`
#![allow(clippy::manual_strip)]

use core::{arch::asm, slice};

use boards::{debug_board_info, get_board};
use log::{debug, info, warn};
use sbi::shutdown;
use task::{kernel_create_process, kernel_create_process_with_args};

use crate::fs_fat32::get_fs;

#[macro_use]
extern crate alloc;

#[macro_use]
mod stdio;
mod allocation;
mod boards;
mod config;
mod driver;
mod ext4;
mod fat32;
mod fs_ext4;
mod fs_fat32;
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

#[no_mangle]
fn main() {
    if option_env!("TEST") == Some("final") {
        test_final();
    } else {
        warn!("No test specified. Running default test.");
        test_preliminary();
    }
}

fn test_preliminary() {
    let test_cases = vec![
        "execve",
        "mmap",
        "munmap",
        "dup",
        "brk",
        "chdir",
        "clone",
        "close",
        "dup2",
        "exit",
        "fork",
        "fstat",
        "getcwd",
        "getdents",
        "getpid",
        "getppid",
        "gettimeofday",
        "mkdir_",
        "mount",
        "open",
        "openat",
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
        "pipe",
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

fn test_final() {
    let time_test = get_fs().root_dir().read_file_as_buf("time_test").unwrap();

    kernel_create_process(&time_test);
    task::run_tasks();

    let busybox = get_fs().root_dir().read_file_as_buf("busybox");

    match busybox {
        Some(busybox) => {
            kernel_create_process_with_args(&busybox, &["busybox", "sh", "./test_all.sh"])
        }
        None => panic!("Busybox not found. Aborting."),
    }

    task::run_tasks();
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

    kernel_init();

    trap::init();
    trap::enable_timer_interrupt();
    timer::set_next_trigger();

    main();

    shutdown(false);
}

fn kernel_init() {
    use crate::sbi::console::UnionConsole;
    use sbi_spec::base::impl_id;

    info!(r#"                          ___  ____  "#);
    info!(r#"  _ __   ___ _   _  __ _ / _ \/ ___|"#);
    info!(r#" | '_ \ / _ \ | | |/ _` | | | \___ \ "#);
    info!(r#" | | | |  __/ |_| | (_| | |_| |___) |"#);
    info!(r#" |_| |_|\___|\__/_|\__/ |\___/|____/ "#);
    info!(r#"                      |_|            "#);

    info!("Hello, world!");

    info!("SBI specification version: {0}", sbi_rt::get_spec_version());

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

    info!("SBI implementation: {0}", sbi_impl);

    let console_type = match UnionConsole::instance() {
        UnionConsole::Legacy(_) => "Legacy",
        UnionConsole::Dbcn(_) => "DBCN",
    };

    info!("Console type: {0}", console_type);

    // board initialization was actually done earlier when we initialized virtual memory
    // since we need to know the memory layout of the board
    let board = get_board();

    mm::frame::init_memory_end(board.memory_end());

    // Only do this when we first initialize the board
    debug_board_info(board.clone());
}

unsafe fn clear_bss() {
    extern "C" {
        fn sbss();
        fn ebss();
    }

    slice::from_mut_ptr_range(sbss as *mut u8..ebss as *mut u8).fill(0);
}
