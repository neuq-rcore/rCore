use super::TaskContext;
use core::arch::asm;

#[naked]
#[no_mangle]
pub unsafe extern "C" fn __switch(
    current_task_ctx_ptr: *mut TaskContext,
    next_task_ctx_ptr: *const TaskContext,
) {
    asm!(
        // store sp
        "sd sp, 8(a0)",
        // store ra
        "sd ra, 0(a0)",
        // store saved registers
        "sd s0, 16(a0)",
        "sd s1, 24(a0)",
        "sd s2, 32(a0)",
        "sd s3, 40(a0)",
        "sd s4, 48(a0)",
        "sd s5, 56(a0)",
        "sd s6, 64(a0)",
        "sd s7, 72(a0)",
        "sd s8, 80(a0)",
        "sd s9, 88(a0)",
        "sd s10, 96(a0)",
        "sd s11, 104(a0)",
        // restore ra
        "ld ra, 0(a1)",
        // restore saved registers
        "ld s0, 16(a1)",
        "ld s1, 24(a1)",
        "ld s2, 32(a1)",
        "ld s3, 40(a1)",
        "ld s4, 48(a1)",
        "ld s5, 56(a1)",
        "ld s6, 64(a1)",
        "ld s7, 72(a1)",
        "ld s8, 80(a1)",
        "ld s9, 88(a1)",
        "ld s10, 96(a1)",
        "ld s11, 104(a1)",
        // restore sp
        "ld sp, 8(a1)",
        "ret",
        options(noreturn)
    );
}
