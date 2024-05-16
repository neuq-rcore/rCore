pub const USER_STACK_SIZE: usize = 4096;
pub const KERNEL_STACK_SIZE: usize = 4096 * 2;

pub const KERNEL_HEAP_SIZE: usize = 0x0080_0000; // 8MB

pub const PAGE_SIZE: usize = 0x1000; // 4KB
pub const PAGE_SIZE_BITS_WIDTH: usize = 12;

pub const MAX_APP_NUM: usize = 4;
pub const APP_BASE_ADDRESS: usize = 0x80400000;
pub const APP_SIZE_LIMIT: usize = 0x20000;

pub const MEMORY_END: usize = 0x8800_0000;

// 跳板函数的地址，虚拟地址空间的最后一页
pub const TRAMPOLINE: usize = usize::MAX - PAGE_SIZE + 1;
pub const TRAP_CONTEXT_VPN: usize = TRAMPOLINE - PAGE_SIZE;

pub use crate::boards::qemu::CLOCK_FREQ;

pub const fn kernel_stack_position(app_id: usize) -> (usize, usize) {
    let top = TRAMPOLINE - app_id * (KERNEL_STACK_SIZE + PAGE_SIZE);
    let bottom = top - KERNEL_STACK_SIZE;
    (bottom, top)
}

