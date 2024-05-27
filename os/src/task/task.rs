use core::cell::Ref;
use core::cell::RefMut;

use alloc::string::String;
use alloc::sync::Arc;
use alloc::sync::Weak;
use alloc::vec::Vec;
use log::info;

use crate::fs::inode::FileDescriptor;
use crate::mm::address::VirtAddr;

use crate::sync::UPSafeCell;
use crate::task::pid::pid_alloc;
use crate::{
    config::TRAP_CONTEXT,
    mm::{kernel_token, MemorySpace, PhysAddr, PhysPageNum, UserSpace},
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
    pub children: Vec<Arc<TaskControlBlock>>,
    pub exit_code: i32,
    pub cwd: String,
    pub heap_pos: usize,
    pub dup_fds: [(isize, isize); 10],
    pub fd_table: Vec<Option<FileDescriptor>>,
}

impl Drop for TaskControlBlock {
    fn drop(&mut self) {
        info!("TaskControlBlock drop: pid={}", self.pid());
    }
}

impl Drop for TaskControlBlockInner {
    fn drop(&mut self) {
        info!("TaskControlBlockInner drop: exit_code={}", self.exit_code);
    }
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
        unsafe { &mut *(&mut self.exclusive_inner().task_ctx as *mut TaskContext) }
    }

    pub fn task_ctx<'a>(&'a self) -> &'a TaskContext {
        unsafe { &*(&self.shared_inner().task_ctx as *const TaskContext) }
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
            cwd: String::from("/"),
            heap_pos: 0,
            dup_fds: [(-100, -100); 10],
            fd_table: vec![
                Some(FileDescriptor::open_stdin()),
                Some(FileDescriptor::open_stdout()),
                Some(FileDescriptor::open_stderr()),
            ],
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

    pub fn exec(&self, elf_bytes: &[u8]) {
        let (new_space, user_sp, entry_point) = UserSpace::from_elf(elf_bytes);
        let trap_cx_ppn = new_space
            .table()
            .translate(VirtAddr::from(TRAP_CONTEXT).into())
            .unwrap()
            .ppn();

        let mut inner = self.exclusive_inner();
        inner.memory_space = new_space;
        inner.trap_ctx_ppn = trap_cx_ppn;
        let trap_ctx = inner.trap_ctx();
        *trap_ctx = TrapContext::app_init_context(
            entry_point,
            user_sp,
            kernel_token(),
            self.kernel_stack.top(),
            trap_handler as usize,
        );
    }

    pub fn fork(self: &Arc<TaskControlBlock>) -> Arc<TaskControlBlock> {
        let mut parent_inner = self.exclusive_inner();
        let child_space = MemorySpace::from_existed_space(&parent_inner.memory_space);

        let trap_ctx_ppn = child_space
            .table()
            .translate(VirtAddr::from(TRAP_CONTEXT).into())
            .unwrap()
            .ppn();

        let pid_handle = pid_alloc();
        assert!(pid_handle.0 != self.pid());

        let kernel_stack = KernelStack::new(&pid_handle);
        let kernel_stack_top = kernel_stack.top();

        let child_task_ctx = TaskContext::goto_trap_return(kernel_stack_top);

        let child_inner = UPSafeCell::new(TaskControlBlockInner {
            task_status: TaskStatus::Ready,
            task_ctx: child_task_ctx,
            memory_space: child_space,
            trap_ctx_ppn,
            base_size: parent_inner.base_size,
            parent: Some(Arc::downgrade(self)),
            children: Vec::new(),
            exit_code: 0,
            cwd: parent_inner.cwd.clone(),
            heap_pos: 0,
            dup_fds: parent_inner.dup_fds.clone(),
            fd_table: parent_inner.fd_table.clone(),
        });

        let child_control_block = Arc::new(TaskControlBlock {
            inner: child_inner,
            pid: pid_handle,
            kernel_stack,
        });

        parent_inner.children.push(child_control_block.clone());

        let child_trap_ctx = child_control_block.exclusive_inner().trap_ctx();
        child_trap_ctx.kernel_sp = kernel_stack_top;

        child_control_block
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
