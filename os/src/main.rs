#![no_main]
#![no_std]
#![feature(
    panic_info_message,
    slice_from_ptr_range,
    naked_functions,
    alloc_error_handler
)]

use core::{
    arch::{asm, global_asm},
    panic, slice,
};

use alloc::string::String;
use sbi::shutdown;

use crate::fat32::Fat32FileSystem;

#[macro_use]
extern crate alloc;

#[macro_use]
pub mod stdio;
mod boards;
mod config;
mod driver;
mod fat32;
mod lang_items;
mod loader;
mod logging;
mod mm;
mod sbi;
mod stack_trace;
mod sync;
pub mod syscall;
pub mod task;
mod timer;
pub mod trap;

// Since we've implemented filesystem, we will soon migrate to test suits from sdcard image
// global_asm!(include_str!("link_app.S"));

fn format_file_size(size: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = 1024 * KB;
    const GB: u64 = 1024 * MB;
    if size < KB {
        format!("{}B", size)
    } else if size < MB {
        format!("{}KB", size / KB)
    } else if size < GB {
        format!("{}MB", size / MB)
    } else {
        format!("{}GB", size / GB)
    }
}

#[no_mangle]
fn main() {
    let fs = Fat32FileSystem::new(0);

    let root_dir = fs.root_dir();

    let only_dir = root_dir.iter().next().unwrap().unwrap().to_dir();

    println!("Files/Dirs in <root/>/riscv64/:");

    for e in only_dir.iter() {
        let e = e.unwrap();
        let name = e.file_name();

        println!("  File/Dir: {}", name);
    }

    // TODO: Implement user space task system
    // task::run_first_task();
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
    loader::load_apps();
    trap::enable_timer_interrupt();
    timer::set_next_trigger();

    debug_env();

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
