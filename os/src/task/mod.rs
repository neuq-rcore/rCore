#[allow(non_snake_case)]
pub mod TaskManager;
mod context;
mod pid;
pub mod processor;
mod switch;

#[allow(clippy::module_inception)]
mod task;

use alloc::sync::Arc;

pub use processor::run_tasks;

pub use context::TaskContext;

use crate::sbi::shutdown;

use self::{
    pid::pid_alloc,
    processor::{schedule, take_current_task},
    task::{TaskControlBlock, TaskStatus},
    TaskManager::{add_task, remove_task},
};

pub fn kernel_create_process(elf_data: &[u8]) {
    let pcb = Arc::new(TaskControlBlock::new(elf_data, pid_alloc()));
    add_task(pcb);
}

/// pid of usertests app in make run TEST=1
pub const IDLE_PID: usize = 0;

pub fn suspend_current_and_run_next() {
    // There must be an application running.
    let task = take_current_task().unwrap();

    // let mut task_inner = task.inner_exclusive_access();
    let task_ctx_ptr = task.task_ctx() as *const _ as *mut _;
    // Change status to Ready
    task.update_status(TaskStatus::Ready);

    // push back to ready queue.
    add_task(task);
    // jump to scheduling cycle
    schedule(task_ctx_ptr);
}

pub fn exit_current_and_run_next(exit_code: i32) {
    // take from Processor
    let task = take_current_task().unwrap();

    let pid = task.pid();

    if pid == IDLE_PID {
        println!(
            "[kernel] Idle process exit with exit_code {} ...",
            exit_code
        );
        if exit_code != 0 {
            //crate::sbi::shutdown(255); //255 == -1 for err hint
            shutdown(true)
        } else {
            //crate::sbi::shutdown(0); //0 for success hint
            shutdown(false)
        }
    }

    // Change status to Zombie
    task.update_status(TaskStatus::Zombie);
    let mut inner = task.exclusive_inner();
    // Record exit code
    inner.exit_code = exit_code;
    // do not move to its parent but under initproc

    for child in inner.children.iter() {
        let mut child_inner = child.exclusive_inner();
        child_inner.parent = None;
    }

    remove_task(pid);

    inner.children.clear();
    // deallocate user space
    inner.memory_space.clear();
    drop(inner);
    // **** release current PCB
    // drop task manually to maintain rc correctly
    drop(task);
    // we do not have to save task context
    let mut _unused = TaskContext::zero_init();
    schedule(&mut _unused as *mut _);
}
