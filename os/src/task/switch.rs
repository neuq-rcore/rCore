use super::TaskContext;
use core::arch::asm;

#[naked]
#[no_mangle]
pub unsafe extern "C" fn __switch(current_task_ctx_ptr: *mut TaskContext, next_task_ctx_ptr: *const TaskContext) -> ! {
    asm!(
        // store sp
        "sd sp, 8(a0)",
        // store ra
        "sd ra, 0(a0)",
        // store saved registers
        "sd s0, 16(a0)",
        "sd s1, 16(a0)",
        "sd s2, 16(a0)",
        "sd s3, 16(a0)",
        "sd s4, 16(a0)",
        "sd s5, 16(a0)",
        "sd s6, 16(a0)",
        "sd s7, 16(a0)",
        "sd s8, 16(a0)",
        "sd s10, 16(a0)",
        "sd s11, 16(a0)",
        // restore ra
        "ld ra, 0(a1)",
        // restore saved registers
        "ld s0, 16(a0)",
        "ld s1, 16(a0)",
        "ld s2, 16(a0)",
        "ld s3, 16(a0)",
        "ld s4, 16(a0)",
        "ld s5, 16(a0)",
        "ld s6, 16(a0)",
        "ld s7, 16(a0)",
        "ld s8, 16(a0)",
        "ld s10, 16(a0)",
        "ld s11, 16(a0)",
        // restore sp
        "ld sp, 8(a1)",
        options(noreturn)
    );
}
