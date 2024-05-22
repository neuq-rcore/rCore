mod context;
mod pid;
mod switch;

#[allow(clippy::module_inception)]
mod task;

pub use pid::tests as pid_tests;

use crate::loader::get_app_elf_data;
use crate::sbi::shutdown;
use crate::sync::UPSafeCell;
use crate::task::pid::pid_alloc;
use crate::{loader::get_num_app, trap::TrapContext};
use alloc::vec::Vec;
use lazy_static::*;
use log::debug;
use switch::__switch;
use task::TaskControlBlock;
use task::TaskStatus;

pub use context::TaskContext;

pub struct TaskManager {
    num_app: usize,
    inner: UPSafeCell<TaskManagerInner>,
}

pub struct TaskManagerInner {
    tasks: Vec<TaskControlBlock>,
    current_task: usize,
}

lazy_static! {
    pub static ref TASK_MANAGER: TaskManager = {
        let num_app = get_num_app();
        debug!("App nums: {}", num_app);
        let mut tasks: Vec<TaskControlBlock> = Vec::new();
        for i in 0..num_app {
            tasks.push(TaskControlBlock::new(
                get_app_elf_data(i).unwrap(),
                pid_alloc(),
            ));
        }
        TaskManager {
            num_app,
            inner: UPSafeCell::new(TaskManagerInner {
                tasks,
                current_task: 0,
            }),
        }
    };
}

impl TaskManager {
    fn run_first_task(&self) -> ! {
        let mut inner = self.inner.exclusive_access();
        let task0 = &mut inner.tasks[0];
        task0.update_status(TaskStatus::Running);
        let next_task_ctx_ptr = task0.task_ctx() as *const TaskContext;
        drop(inner);
        let mut _unused = TaskContext::zero_init();
        unsafe {
            __switch(&mut _unused as *mut TaskContext, next_task_ctx_ptr);
        }
        panic!("unreachable in run_first_task!");
    }

    fn mark_current_suspended(&self) {
        let mut inner = self.inner.exclusive_access();
        let current = inner.current_task;
        inner.tasks[current].update_status(TaskStatus::Ready);
    }

    fn mark_current_exited(&self) {
        let mut inner = self.inner.exclusive_access();
        let current = inner.current_task;
        inner.tasks[current].update_status(TaskStatus::Exited);
    }

    fn find_next_task(&self) -> Option<usize> {
        let inner = self.inner.exclusive_access();
        let current = inner.current_task;
        (current + 1..current + self.num_app + 1)
            .map(|id| id % self.num_app)
            .find(|id| inner.tasks[*id].status() == TaskStatus::Ready)
    }

    fn run_next_task(&self) {
        if let Some(next) = self.find_next_task() {
            let mut inner = self.inner.exclusive_access();
            let current = inner.current_task;
            inner.tasks[next].update_status(TaskStatus::Running);
            inner.current_task = next;
            let current_task_ctx_ptr = inner.tasks[current].task_ctx_mut() as *mut TaskContext;
            let next_task_ctx_ptr = inner.tasks[next].task_ctx() as *const TaskContext;
            drop(inner);
            unsafe {
                __switch(current_task_ctx_ptr, next_task_ctx_ptr);
            }
        } else {
            println!("All applications completed!");
            shutdown(false);
        }
    }

    fn get_current_token(&self) -> usize {
        let inner = self.inner.shared_access();
        let current = inner.current_task;
        inner.tasks[current].token()
    }

    fn get_current_trap_cx(&self) -> &mut TrapContext {
        let mut inner = self.inner.exclusive_access();
        let current = inner.current_task;
        inner.tasks[current].trap_ctx_mut()
    }
}

pub fn current_task() -> &'static TaskControlBlock {
    let taskmgr = TASK_MANAGER.inner.exclusive_access();
    let task_id = taskmgr.current_task;
    unsafe {
        & *(&taskmgr.tasks[task_id] as *const TaskControlBlock)
    }
}

pub fn current_user_token() -> usize {
    TASK_MANAGER.get_current_token()
}

pub fn current_trap_cx() -> &'static mut TrapContext {
    TASK_MANAGER.get_current_trap_cx()
}

pub fn run_first_task() {
    TASK_MANAGER.run_first_task();
}

fn run_next_task() {
    TASK_MANAGER.run_next_task();
}

fn mark_current_suspended() {
    TASK_MANAGER.mark_current_suspended();
}

fn mark_current_exited() {
    TASK_MANAGER.mark_current_exited();
}

pub fn suspend_current_and_run_next() {
    mark_current_suspended();
    run_next_task();
}

pub fn exit_current_and_run_next() {
    mark_current_exited();
    run_next_task();
}
