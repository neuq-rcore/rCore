use log::error;

use crate::sbi::shutdown;
use crate::stack_trace::print_stack_trace;
use core::panic::PanicInfo;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    let msg = info.message();

    if let Some(location) = info.location() {
        error!(
            "Panicked at {}:{} {}",
            location.file(),
            location.line(),
            msg
        );
    } else {
        error!("Panicked: {}", msg);
    }
    unsafe { print_stack_trace() }
    shutdown(true)
}
