#![no_main]
#![no_std]
#![feature(
    panic_info_message,
    slice_from_ptr_range,
    naked_functions,
    alloc_error_handler,
    vec_into_raw_parts
)]

use core::{
    arch::{asm, global_asm},
    slice,
};

use alloc::{string::String, vec::Vec};
use fatfs::Read;
use log::{debug, info};
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
global_asm!(include_str!("link_app.S"));

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

    let entries = only_dir.iter();

    debug!("Filesystem initialized.");

    let test_cases = vec!["write"];

    for entry in entries {
        if entry.is_err() {
            continue;
        }

        let entry = entry.unwrap();
        if entry.is_dir() {
            continue;
        }

        let file_name = entry.file_name();

        for name in test_cases.iter() {
            if file_name != *name {
                continue;
            }

            let file_len = entry.len() as usize;
            
            let mut buf: Vec<u8> = Vec::with_capacity(file_len);
            unsafe {
                buf.set_len(file_len);
            }

            let slice = buf.as_mut();
            let mut file = entry.to_file();

            file.read_exact(slice).unwrap();

            let id = loader::load_app(buf);
            loader::add_pending_task(id).unwrap();
        }
    }

    loader::load_apps();

    debug!("Running user app `write` from sdcard.img");

    task::run_first_task();
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
