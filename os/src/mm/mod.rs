pub mod address;
pub mod frame;
pub mod heap;
pub mod page;
pub mod memory;

use self::memory::KernelSpace;

pub fn init() {
    heap::init();
    frame::init();

    KernelSpace::activate();
}