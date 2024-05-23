use core::cell::Ref;
use core::cell::RefMut;

use alloc::sync::Weak;
use alloc::sync::Arc;
use alloc::vec::Vec;

use crate::mm::address::VirtAddr;

use crate::sync::UPSafeCell;
use crate::{
    config::TRAP_CONTEXT,
    mm::{
        kernel_token, MemorySpace, PhysAddr, PhysPageNum, UserSpace,
    },
    trap::{trap_handler, TrapContext},
};

use super::pid::{KernelStack, PidHandle};
use super::TaskContext;

pub struct TaskControlBlock {
    // immutable
    pub pid: PidHandle,
    pub kernel_stack: KernelStack,
    // mutable
    inner: UPSafeCell<TaskControlBlockInner>,    
}

pub struct TaskControlBlockInner {
    pub task_status: TaskStatus,
    pub task_ctx: TaskContext,
    pub memory_space: MemorySpace,
    pub trap_ctx_ppn: PhysPageNum,
    pub base_size: usize,
    pub parent: Option<Weak<TaskControlBlock>>,
    pub children: Vec<Arc<TaskControlBlockInner>>,
    pub exit_code: i32,
}

impl TaskControlBlockInner {
    pub fn trap_ctx(&self) -> &'static mut TrapContext {
        let pa: PhysAddr = self.trap_ctx_ppn.into();
        unsafe { (pa.0 as *mut TrapContext).as_mut().unwrap() }
    }

    pub fn token(&self) -> usize {
        self.memory_space.token()
    }

    pub fn status(&self) -> TaskStatus {
        self.task_status
    }

    pub fn is_zombie(&self) -> bool {
        self.status() == TaskStatus::Zombie
    }
}

impl TaskControlBlock {
    pub fn exclusive_inner(&self) -> RefMut<TaskControlBlockInner> {
        self.inner.exclusive_access()
    }

    pub fn shared_inner(&self) -> Ref<TaskControlBlockInner> {
        self.inner.shared_access()
    }

    pub fn trap_ctx(&self) -> &'static TrapContext {
        self.shared_inner().trap_ctx()
    }

    pub fn trap_ctx_mut(&mut self) -> &'static mut TrapContext {
        self.exclusive_inner().trap_ctx()
    }

    pub fn token(&self) -> usize {
        self.shared_inner().token()
    }

    pub fn status(&self) -> TaskStatus {
        self.shared_inner().task_status
    }

    pub fn is_zombie(&self) -> bool {
        self.status() == TaskStatus::Zombie
    }

    pub fn update_status(&self, new_status: TaskStatus) {
        self.exclusive_inner().task_status = new_status
    }

    pub fn task_ctx_mut<'a>(&'a mut self) -> &'a mut TaskContext {
        unsafe {
            &mut *(&mut self.exclusive_inner().task_ctx as *mut TaskContext)
        }
    }

    pub fn task_ctx<'a>(&'a self) -> &'a TaskContext {
        unsafe {
            &*(&self.shared_inner().task_ctx as *const TaskContext)
        }
    }
}

impl TaskControlBlock {
    pub fn new(elf_bytes: &[u8], pid: PidHandle) -> Self {
        let (user_space, user_sp, entry_point) = UserSpace::from_elf(elf_bytes);

        let trap_ctx_ppn = user_space
            .table()
            .translate(VirtAddr::from(TRAP_CONTEXT).into())
            .unwrap()
            .ppn();

        let kernel_stack = KernelStack::new(&pid);
        let kernel_stack_top = kernel_stack.top();

        let control_block_inner = TaskControlBlockInner {
            task_status: TaskStatus::Ready,
            task_ctx: TaskContext::goto_trap_return(kernel_stack_top),
            memory_space: user_space,
            trap_ctx_ppn,
            base_size: user_sp,
            parent: None,
            children: Vec::new(),
            exit_code: 0,
        };

        let kernel_token = kernel_token();

        let trap_ctx = control_block_inner.trap_ctx();
        *trap_ctx = TrapContext::app_init_context(
            entry_point,
            user_sp,
            kernel_token,
            kernel_stack_top,
            trap_handler as usize,
        );

        let control_block = TaskControlBlock {
            inner: UPSafeCell::new(control_block_inner),
            pid,
            kernel_stack,
        };

        control_block
    }

    pub fn pid(&self) -> usize {
        self.pid.0
    }

    pub fn exec(&mut self, elf_bytes: &[u8]) -> ! {
        unimplemented!()
    }

    pub fn fork(&mut self) -> TaskControlBlock {
        unimplemented!();
    }
}

#[derive(Copy, Clone, PartialEq)]
pub enum TaskStatus {
    UnInit,
    Ready,
    Running,
    Exited,
    Zombie,
}
