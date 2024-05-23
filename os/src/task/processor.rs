use alloc::sync::Arc;
use lazy_static::lazy_static;

use crate::{sync::UPSafeCell, trap::TrapContext};

use super::TaskManager::fetch_task;
use super::{switch::__switch, task::{TaskControlBlock, TaskStatus}, TaskContext};

pub struct Processor {
    current: Option<Arc<TaskControlBlock>>,
    idle_task_cx: TaskContext,
}

impl Processor {
    fn new() -> Self {
        Processor {
            current: None,
            idle_task_cx: TaskContext::zero_init(),
        }
    }

    pub fn take_current(&mut self) -> Option<Arc<TaskControlBlock>> {
        self.current.take()
    }

    pub fn current(&self) -> Option<Arc<TaskControlBlock>> {
        self.current.as_ref().map(|task| Arc::clone(task))
    }
}


impl Processor {
    fn get_idle_task_cx_ptr(&mut self) -> *mut TaskContext {
        &mut self.idle_task_cx as *mut _
    }
}

lazy_static! {
    pub static ref PROCESSOR: UPSafeCell<Processor> = UPSafeCell::new(Processor::new());
}

pub fn take_current_task() -> Option<Arc<TaskControlBlock>> {
    PROCESSOR.exclusive_access().take_current()
}

pub fn current_task() -> Option<Arc<TaskControlBlock>> {
    PROCESSOR.exclusive_access().current()
}

pub fn current_user_token() -> usize {
    let task = current_task().unwrap();

    task.token()
}

pub fn current_trap_ctx() -> &'static mut TrapContext {
    current_task().unwrap().exclusive_inner().trap_ctx()
}

pub fn run_tasks() {
    while let Some(task) = fetch_task() {
        let mut processor = PROCESSOR.exclusive_access();

        let idle_task_cx_ptr = processor.get_idle_task_cx_ptr();
            // access coming task TCB exclusively
            let next_task_cx_ptr = task.task_ctx() as *const TaskContext;
            task.update_status(TaskStatus::Running);
            // stop exclusively accessing coming task TCB manually
            processor.current = Some(task.clone());
            // stop exclusively accessing processor manually
            drop(processor);
            unsafe {
                __switch(
                    idle_task_cx_ptr,
                    next_task_cx_ptr,
                );
            }
    }
}

pub fn schedule(switched_task_cx_ptr: *mut TaskContext) {
    let mut processor = PROCESSOR.exclusive_access();
    let idle_task_cx_ptr = processor.get_idle_task_cx_ptr();
    drop(processor);
    unsafe {
        __switch(
            switched_task_cx_ptr,
            idle_task_cx_ptr,
        );
    }
}
