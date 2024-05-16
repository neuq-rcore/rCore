use crate::{
    config::{kernel_stack_position, TRAP_CONTEXT_VPN},
    mm::{
        kernel_token, MapPermission, MemorySpace, PhysAddr, PhysPageNum, UserSpace, VirtPageNum,
        KERNEL_SPACE,
    },
    trap::{trap_handler, TrapContext},
};

use super::TaskContext;

pub struct TaskControlBlock {
    pub task_status: TaskStatus,
    pub task_ctx: TaskContext,
    pub memory_space: MemorySpace,
    pub trap_ctx_ppn: PhysPageNum,
    pub base_size: usize,
}

impl TaskControlBlock {
    pub fn new(elf_bytes: &[u8], app_id: usize) -> Self {
        let (user_space, user_sp, entry_point) = UserSpace::from_elf(elf_bytes);

        // TODO: Don't know why unwrap fails.
        let trap_ctx_ppn = user_space
            .table()
            .translate(VirtPageNum::from(TRAP_CONTEXT_VPN).into())
            .unwrap()
            .ppn();

        let (kernel_stack_bottom, kernel_stack_top) = kernel_stack_position(app_id);

        KERNEL_SPACE.exclusive_access().insert_framed_area(
            kernel_stack_bottom.into(),
            kernel_stack_top.into(),
            MapPermission::R | MapPermission::W,
        );

        let task_control_block = Self {
            task_status: TaskStatus::Ready,
            task_ctx: TaskContext::goto_trap_return(kernel_stack_top),
            memory_space: user_space,
            trap_ctx_ppn,
            base_size: user_sp,
        };

        let kernel_token = kernel_token();

        let trap_ctx = task_control_block.trap_ctx();
        *trap_ctx = TrapContext::app_init_context(
            entry_point,
            user_sp,
            kernel_token,
            kernel_stack_top,
            trap_handler as usize,
        );

        task_control_block
    }

    pub fn trap_ctx(&self) -> &'static mut TrapContext {
        let pa: PhysAddr = self.trap_ctx_ppn.into();
        unsafe { (pa.0 as *mut TrapContext).as_mut().unwrap() }
    }
}

#[derive(Copy, Clone, PartialEq)]
pub enum TaskStatus {
    UnInit,
    Ready,
    Running,
    Exited,
}
