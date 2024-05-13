use crate::config::KERNEL_HEAP_SIZE;
use buddy_system_allocator::LockedHeap;
use log::debug;

#[link_section = ".bss.heap"]
static mut HEAP: [u8; KERNEL_HEAP_SIZE] = [0; KERNEL_HEAP_SIZE];

#[global_allocator]
static HEAP_ALLOCATOR: LockedHeap<30> = LockedHeap::empty();

pub fn init() {
    unsafe {
        HEAP_ALLOCATOR
            .lock()
            .init(HEAP.as_ptr() as usize, KERNEL_HEAP_SIZE);

        debug!(
            "heap: init end, range: [{:#x}, {:#x})",
            HEAP.as_ptr() as usize,
            HEAP.as_ptr() as usize + KERNEL_HEAP_SIZE
        );
    }
}

#[alloc_error_handler]
fn alloc_error_handler(layout: core::alloc::Layout) -> ! {
    panic!("[Kernel] Allocation error, layout: {:?}", layout)
}
