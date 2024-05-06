mod system;
mod qemu;

pub mod console;
pub use system::shutdown as shutdown;
pub use system::set_timer as set_timer;