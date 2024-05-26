use alloc::collections::VecDeque;
use alloc::sync::Arc;
use lazy_static::lazy_static;

use crate::sync::UPSafeCell;

use super::task::TaskControlBlock;

pub struct TaskManager {
    ready_queue: VecDeque<Arc<TaskControlBlock>>,
    waiting_queue: VecDeque<(Arc<TaskControlBlock>, Arc<dyn Fn() -> bool>)>,
}

lazy_static! {
    pub static ref TASK_MANAGER: UPSafeCell<TaskManager> = UPSafeCell::new(TaskManager::new());
}

unsafe impl Send for TaskManager {}
unsafe impl Sync for TaskManager {}

impl TaskManager {
    pub fn new() -> Self {
        Self {
            ready_queue: VecDeque::new(),
            waiting_queue: VecDeque::new(),
        }
    }

    pub fn add(&mut self, task: Arc<TaskControlBlock>) {
        self.ready_queue.push_back(task);
    }

    pub fn schedule(&mut self) {
        self.waiting_queue.retain(|taskdesc| {
            if taskdesc.1() {
                self.ready_queue.push_front(Arc::clone(&taskdesc.0));
                false
            } else {
                true
            }
        });
    }

    pub fn fetch(&mut self) -> Option<Arc<TaskControlBlock>> {
        self.schedule();
        self.ready_queue.pop_front()
    }

    pub fn remove(&mut self, pid: usize) {
        self.ready_queue.retain(|task| task.pid() != pid);
    }

    pub fn add_to_waiting(
        &mut self,
        task: Arc<TaskControlBlock>,
        assertion: Arc<dyn Fn() -> bool>,
    ) {
        self.remove(task.clone().pid());
        self.waiting_queue.push_back((task, assertion));
    }
}

pub fn add_to_waiting(task: Arc<TaskControlBlock>, assertion: Arc<dyn Fn() -> bool>) {
    TASK_MANAGER
        .exclusive_access()
        .add_to_waiting(task, assertion);
}

pub fn remove_task(pid: usize) {
    TASK_MANAGER.exclusive_access().remove(pid);
}

pub fn add_task(task: Arc<TaskControlBlock>) {
    TASK_MANAGER.exclusive_access().add(task);
}

pub fn fetch_task() -> Option<Arc<TaskControlBlock>> {
    TASK_MANAGER.exclusive_access().fetch()
}
