use alloc::vec::Vec;

use crate::config::*;
use crate::trap::TrapContext;
use core::arch::asm;

#[repr(align(4096))]
#[derive(Copy, Clone)]
struct KernelStack {
    data: [u8; KERNEL_STACK_SIZE],
}

#[repr(align(4096))]
#[derive(Copy, Clone)]
struct UserStack {
    data: [u8; USER_STACK_SIZE],
}

static KERNEL_STACK: [KernelStack; MAX_APP_NUM] = [KernelStack {
    data: [0; KERNEL_STACK_SIZE],
}; MAX_APP_NUM];

static USER_STACK: [UserStack; MAX_APP_NUM] = [UserStack {
    data: [0; USER_STACK_SIZE],
}; MAX_APP_NUM];

impl KernelStack {
    fn get_sp(&self) -> usize {
        self.data.as_ptr() as usize + KERNEL_STACK_SIZE
    }

    pub fn push_context(&self, trap_ctx: TrapContext) -> usize {
        let trap_ctx_ptr =
            (self.get_sp() - core::mem::size_of::<TrapContext>()) as *mut TrapContext;
        unsafe {
            *trap_ctx_ptr = trap_ctx;
        }
        trap_ctx_ptr as usize
    }
}

impl UserStack {
    fn get_sp(&self) -> usize {
        self.data.as_ptr() as usize + USER_STACK_SIZE
    }
}

fn get_base_i(app_id: usize) -> usize {
    APP_BASE_ADDRESS + app_id * APP_SIZE_LIMIT
}

pub fn get_num_app() -> usize {
    unsafe { TASK_APPID.len() }
}

static mut APP_ELF_DATA: Vec<Vec<u8>> = Vec::new();
static mut TASK_APPID: Vec<usize> = Vec::new();

// Add an app and return the app_id
pub fn load_app(bytes: Vec<u8>) -> usize {
    let app_id = unsafe { APP_ELF_DATA.len() };
    unsafe {
        APP_ELF_DATA.push(bytes);
    }
    app_id
}

pub fn add_pending_task(app_id: usize) -> Result<(), ()> {
    if unsafe { APP_ELF_DATA.len() <= app_id } || unsafe { TASK_APPID.len() >= MAX_APP_NUM } {
        Err(())
    } else {
        unsafe {
            TASK_APPID.push(app_id);
        }
        Ok(())
    }
}

pub fn load_apps() {
    let num_app = get_num_app();

    for id in 0..num_app {
        let base_i = get_base_i(id);

        (base_i..base_i + APP_SIZE_LIMIT)
            .for_each(|addr| unsafe { (addr as *mut u8).write_volatile(0) });

        let src = get_app_elf_data(id).unwrap();
        let dst = unsafe { core::slice::from_raw_parts_mut(base_i as *mut u8, src.len()) };
        dst.copy_from_slice(src);
    }

    unsafe {
        asm!("fence.i");
    }
}

pub fn get_app_elf_data(app_id: usize) -> Option<&'static [u8]> {
    match unsafe { TASK_APPID.contains(&app_id) } {
        false => None,
        true => {
            let data = unsafe { APP_ELF_DATA[app_id].as_slice() };

            Some(&data)
        }
    }
}
