pub fn set_timer(timer: usize) {
    sbi_rt::set_timer(timer as _);
}


pub fn shutdown(failure: bool) -> ! {
    use super::qemu::{QEMUExit, QEMU_EXIT_HANDLE};
    
    match failure {
        true => QEMU_EXIT_HANDLE.exit_failure(),
        false => QEMU_EXIT_HANDLE.exit_success(),
    }
}
