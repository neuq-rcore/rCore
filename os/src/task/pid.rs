use alloc::collections::BTreeSet;
use lazy_static::lazy_static;
use log::*;

use crate::{config::{KERNEL_STACK_SIZE, PAGE_SIZE, TRAMPOLINE}, mm::{MapPermission, VirtAddr, KERNEL_SPACE}, sync::UPSafeCell};

#[derive(Debug, PartialEq)]
pub struct PidHandle(pub usize);

impl Eq for PidHandle {}

impl PidHandle {
    fn new(pid: usize) -> Self {
        PidHandle(pid)
    }
}

impl From<usize> for PidHandle {
    fn from(value: usize) -> Self {
        Self::new(value)
    }
}

impl From<PidHandle> for usize {
    fn from(value: PidHandle) -> Self {
        value.0
    }
}

struct PidManager {
    current: usize,
    recycled: BTreeSet<usize>,
}

impl PidManager {
    fn new() -> Self {
        PidManager {
            // 0 is reserved for internal use
            current: 1,
            recycled: BTreeSet::new(),
        }
    }

    fn allocate(&mut self) -> PidHandle {
        PidHandle::new(match self.recycled.is_empty() {
            true => {
                let pid = self.current;
                self.current += 1;
                pid
            }
            false => {
                let pid = *self.recycled.iter().next().unwrap();
                self.recycled.remove(&pid);
                pid
            }
        })
    }

    fn deallocate(&mut self, pid: &PidHandle) {
        // Pid must be allocated
        assert!(pid.0 < self.current);

        // Pid must be valid
        assert!(pid.0 > 0);

        debug!("Deallocating {}", pid.0);
        let inserted = self.recycled.insert(pid.0);

        assert!(inserted, "pid already deallocated");
    }

    // Call this when there are no processes left
    pub fn try_gc_recycled(&mut self) {
        // When there is no pid allocated, we can reset the current pid to 1

        // Check that if all pids in self.recycled are continous,
        // and the last one is self.current - 1

        let mut iter = self.recycled.iter();

        match iter.next() {
            None => return, // fast path
            Some(&pid) => {
                let mut curr = pid;

                while let Some(&pid) = iter.next() {
                    if pid != curr + 1 {
                        return;
                    }

                    curr = pid;
                }

                // We could gc the recycled pids as all of them are deallocated.
                if curr == self.current - 1 {
                    // it's actually safe to gc the recycled pids
                    self.force_reset();
                } else {
                    warn!("Recycled pids are not continous");
                }
            }
        }
    }

    fn force_reset(&mut self) {
        self.current = 1; // 0 is reserved for internal use
        self.recycled.clear();
    }
}

pub mod tests {
    use super::*;

    #[allow(unused)]
    pub fn test_all() {
        test_allocate_pid();
        test_deallocate_pid();
        test_allocate_after_deallocate_last_current();
        test_reallocate_deallocated_pid();
        test_gc();

        // This test should panic so we run it last
        test_deallocate_invalid_pid();
    }

    pub fn test_allocate_pid() {
        PID_ALLOCATOR.exclusive_access().force_reset();

        debug!("Running `test_allocate_pid`");
        let pid1 = super::pid_alloc();
        assert_eq!(pid1.0, 1);

        let pid2 = super::pid_alloc();
        assert_eq!(pid2.0, 2);
    }

    pub fn test_deallocate_pid() {
        PID_ALLOCATOR.exclusive_access().force_reset();

        debug!("Running `test_deallocate_pid`");
        let pid1 = super::pid_alloc();
        let pid2 = super::pid_alloc();
        drop(pid1);
        drop(pid2);
    }

    pub fn test_reallocate_deallocated_pid() {
        PID_ALLOCATOR.exclusive_access().force_reset();

        debug!("Running `test_reallocate_deallocated_pid`");
        let pid1 = super::pid_alloc();
        let pid2 = super::pid_alloc();
        drop(pid1);
        drop(pid2);

        let pid3 = super::pid_alloc();
        assert_eq!(pid3.0, 1);

        let pid4 = super::pid_alloc();
        assert_eq!(pid4.0, 2);
    }

    pub fn test_allocate_after_deallocate_last_current() {
        PID_ALLOCATOR.exclusive_access().force_reset();

        debug!("Running `test_allocate_after_deallocate_last_current`");
        let _pid1 = super::pid_alloc();
        let pid2 = super::pid_alloc();
        drop(pid2);

        let pid3 = super::pid_alloc();
        assert_eq!(pid3.0, 2);
    }

    pub fn test_gc() {
        debug!("Running `test_gc`");

        PID_ALLOCATOR.exclusive_access().force_reset();

        {
            let _p1 = super::pid_alloc();
            let _p2 = super::pid_alloc();
            
            assert_eq!(_p2.0, 2);

            // drop pids
        }

        let _p3 = super::pid_alloc();
        assert_eq!(_p3.0, 1);

        drop(_p3);

        let mut allocator = PID_ALLOCATOR.exclusive_access();

        allocator.try_gc_recycled();

        assert!(allocator.recycled.is_empty());
        assert_eq!(allocator.current, 1);

        // allocator was drop first
    }

    #[should_panic]
    pub fn test_deallocate_invalid_pid() {
        PID_ALLOCATOR.exclusive_access().force_reset();

        debug!("Running `test_deallocate_invalid_pid`");
        let pid1 = PidHandle(100);

        drop(pid1)
    }
}

lazy_static! {
    static ref PID_ALLOCATOR: UPSafeCell<PidManager> =
        UPSafeCell::new(PidManager::new());
}

pub fn pid_alloc() -> PidHandle {
    PID_ALLOCATOR.exclusive_access().allocate()
}

pub fn try_gc_pid_allocator() {
    PID_ALLOCATOR.exclusive_access().try_gc_recycled();
}

impl Drop for PidHandle {
    fn drop(&mut self) {
        PID_ALLOCATOR.exclusive_access().deallocate(self);
    }
}

pub struct KernelStack {
    pid: usize,
}

/// Return (bottom, top) of a kernel stack in kernel space.
pub fn kernel_stack_position(app_id: usize) -> (usize, usize) {
    let top = TRAMPOLINE - app_id * (KERNEL_STACK_SIZE + PAGE_SIZE);
    let bottom = top - KERNEL_STACK_SIZE;
    (bottom, top)
}

impl KernelStack {
    pub fn new(pid_handle: &PidHandle) -> Self {
        let pid = pid_handle.0;
        let (kernel_stack_bottom, kernel_stack_top) = kernel_stack_position(pid);

        KERNEL_SPACE
            .exclusive_access()
            .insert_framed_area(
                kernel_stack_bottom.into(),
                kernel_stack_top.into(),
                MapPermission::R | MapPermission::W,
            );

        KernelStack {
            pid: pid_handle.0,
        }
    }
    pub fn push_on_top<T>(&self, value: T) -> *mut T where
        T: Sized, {
        let kernel_stack_top = self.top();
        let ptr_mut = (kernel_stack_top - core::mem::size_of::<T>()) as *mut T;
        unsafe { *ptr_mut = value; }
        ptr_mut
    }
    pub fn top(&self) -> usize {
        let (_, kernel_stack_top) = kernel_stack_position(self.pid);
        kernel_stack_top
    }
}

impl Drop for KernelStack {
    fn drop(&mut self) {
        let (kernel_stack_bottom, _) = kernel_stack_position(self.pid);
        let kernel_stack_bottom_va: VirtAddr = kernel_stack_bottom.into();
        KERNEL_SPACE
            .exclusive_access()
            .remove_area_with_start_vpn(kernel_stack_bottom_va.into());
    }
}
