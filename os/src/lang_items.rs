use log::error;

use crate::sbi::shutdown;
use crate::stack_trace::print_stack_trace;
use core::panic::PanicInfo;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    if let Some(location) = info.location() {
        error!(
            "Panicked at {}:{} {}",
            location.file(),
            location.line(),
            info.message().unwrap()
        );
    } else {
        error!("Panicked: {}", info.message().unwrap());
    }
    unsafe { print_stack_trace() }
    shutdown(true)
}
