use sbi_rt::{system_reset, NoReason, Shutdown, SystemFailure};

pub fn set_timer(timer: usize) {
    sbi_rt::set_timer(timer as _);
}

pub fn shutdown(failure: bool) -> ! {
    use super::qemu::{IQEMUExit, QEMU_EXIT_HANDLE};

    match option_env!("IS_CI_ENVIRONMENT") {
        Some(_) => {
            // QEMU_EXIT can mark a CI workflow as failed or successful
            match failure {
                true => QEMU_EXIT_HANDLE.exit_failure(),
                false => QEMU_EXIT_HANDLE.exit_success(),
            }
        }
        None => {
            match failure {
                true => system_reset(Shutdown, SystemFailure),
                false => system_reset(Shutdown, NoReason),
            };

            unreachable!();
        }
    }
}
