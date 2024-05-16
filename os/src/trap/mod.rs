mod context;

use crate::config::{TRAMPOLINE, TRAP_CONTEXT};
use crate::syscall::syscall;
use crate::task::{current_user_token, exit_current_and_run_next, suspend_current_and_run_next};
use crate::timer::set_next_trigger;
pub use context::TrapContext;
use core::arch::asm;
use log::{debug, warn};
use riscv::register::{
    mtvec::TrapMode,
    scause::{self, Exception, Interrupt, Trap},
    sie, stval, stvec,
};

pub fn init() {
    set_user_trap();
}

fn set_user_trap() {
    debug!("Entering user trap mode");
    unsafe {
        stvec::write(__snap_trap as usize, TrapMode::Direct);
    }
}

fn set_kernel_trap() -> KernelTrapContext {
    debug!("Entering kernel trap mode");
    unsafe { stvec::write(on_kernel_trap as usize, TrapMode::Direct) }

    KernelTrapContext
}

pub fn enable_timer_interrupt() {
    unsafe {
        sie::set_stimer();
    }
}

#[no_mangle]
pub fn trap_handler(ctx: &mut TrapContext) -> &mut TrapContext {
    let scause = scause::read();
    let stval = stval::read();

    let _kernel_ctx = KernelTrapContext::enter();

    match scause.cause() {
        Trap::Interrupt(Interrupt::SupervisorTimer) => {
            set_next_trigger();
            suspend_current_and_run_next();
        }
        Trap::Exception(Exception::UserEnvCall) => {
            ctx.sepc += 4;
            ctx.x[10] = syscall(ctx.x[17], [ctx.x[10], ctx.x[11], ctx.x[12]]) as usize;
        }
        Trap::Exception(Exception::StoreFault) | Trap::Exception(Exception::StorePageFault) => {
            warn!(
                "PageFault(0x{:016x}) in application, kernel killed it.",
                stval
            );
            exit_current_and_run_next();
        }
        Trap::Exception(Exception::IllegalInstruction) => {
            warn!("IllegalInstruction in application, kernel killed it.");
            exit_current_and_run_next();
        }
        _ => {
            panic!(
                "Unsupported trap {:?}, stval = {:#x}!",
                scause.cause(),
                stval
            );
        }
    }

    // drop _kernel_ctx to restore user trap
    ctx
}

struct KernelTrapContext;

impl KernelTrapContext {
    fn enter() -> Self {
        set_kernel_trap()
    }
}

impl Drop for KernelTrapContext {
    fn drop(&mut self) {
        trap_return()
    }
}

#[allow(unreachable_code)]
pub fn trap_return() -> ! {
    set_user_trap();
    let trap_ctx = TRAP_CONTEXT;
    let user_satp = current_user_token();

    let restore_va = __restore_snap as usize - __snap_trap as usize + TRAMPOLINE;

    unsafe {
        asm!(
            "fence.i",
            "jr {restore_va}",
            restore_va = in(reg) restore_va,
            in("a0") trap_ctx,
            in("a1") user_satp,
            options(noreturn)
        );
    }

    unreachable!("Unreachable in back_to_user!");
}

#[naked]
#[no_mangle]
unsafe extern "C" fn on_kernel_trap() -> ! {
    // TODO:
    // 1. Switch to kernel stack
    asm!("call kernel_trap_intenral", options(noreturn))
}

#[no_mangle]
fn kernel_trap_intenral() -> ! {
    panic!("Exception from kernel!")
}

#[naked]
#[no_mangle]
#[link_section = ".text.trampoline"]
pub unsafe extern "C" fn __snap_trap() -> ! {
    /*
       |   x0   |  <- sp
       |   x1   |  <- sp + 8
       |   ...  |
       | sstatus|
       |  sepc  |
       | ktoken |
       |  ksp   |
       |  trap  |
       +--------+
    */

    /*
       x0: zero（硬编码为零，不能被写入）
       x1: ra（返回地址）
       x2: sp（堆栈指针）
       x3: gp（全局指针）
       x4: tp（线程指针）
       x5 - x7: t0 - t2（临时/调用者保存）
       x8: s0/fp（保存的寄存器/帧指针）
       x9: s1（保存的寄存器）
       x10 - x11: a0 - a1（函数参数/返回值）
       x12 - x17: a2 - a7（函数参数）
       x18 - x27: s2 - s11（保存的寄存器）
       x28 - x31: t3 - t6（临时/调用者保存）
    */

    asm!(
        // Make sp -> TrapContext
        // sscratch -> User stack
        "csrrw sp, sscratch, sp",
        // Save all registers
        // We will not handle x0, but still leave a room for it
        // "sd x0, 0(sp)",
        "sd ra, 8(sp)", // x1
        // "sd sp, 16(sp)", // 'sp' was broken, we will save with temp register later
        "sd gp, 24(sp)",   // x3
        "sd tp, 32(sp)",   // x4
        "sd t0, 40(sp)",   // x5
        "sd t1, 48(sp)",   // x6
        "sd t2, 56(sp)",   // x7
        "sd s0, 64(sp)",   // x8 aka. fp
        "sd s1, 72(sp)",   // x9
        "sd a0, 80(sp)",   // x10 param0/ret
        "sd a1, 88(sp)",   // x11 param1
        "sd a2, 96(sp)",   // x12
        "sd a3, 104(sp)",  // x13
        "sd a4, 112(sp)",  // x14
        "sd a5, 120(sp)",  // x15
        "sd a6, 128(sp)",  // x16
        "sd a7, 136(sp)",  // x17
        "sd s2, 144(sp)",  // x18
        "sd s3, 152(sp)",  // x19
        "sd s4, 160(sp)",  // x20
        "sd s5, 168(sp)",  // x21
        "sd s6, 176(sp)",  // x22
        "sd s7, 184(sp)",  // x23
        "sd s8, 192(sp)",  // x24
        "sd s9, 200(sp)",  // x25
        "sd s10, 208(sp)", // x26
        "sd s11, 216(sp)", // x27
        "sd t3, 224(sp)",  // x28
        "sd t4, 232(sp)",  // x29
        "sd t5, 240(sp)",  // x30
        "sd t6, 248(sp)",  // x31
        // Since we've saved all temp registers, we can now save sp and other privileged registers with them
        "csrr t0, sstatus",
        "sd t0, 256(sp)", // 32 * 8 = 256
        //
        "csrr t0, sepc",  // PC when trap happened
        "sd t0, 264(sp)", // 33 * 8 = 264
        //
        "csrr t0, sscratch", // Previous sp, we've swaped it with sp at the beginning
        "sd t0, 16(sp)",     // x2
        // Snap end, load kernel registers and jump to kernel trap handler
        "ld t1, 288(sp)", // Address of `trap_handler`
        "ld sp, 280(sp)", // Kernel stack
        "ld t0, 272(sp)", // kernel_token(root ppn)
        "csrw satp, t0",
        // Clear tlbs as we are entering new context(Kernel Space)
        "sfence.vma",
        // Don't use `call trap_handler` here
        "jr t1",
        // The trap_handler never returns
        options(noreturn)
    )
}

#[naked]
#[no_mangle]
#[link_section = ".text.trampoline"]
pub unsafe extern "C" fn __restore_snap(/*context_snap: *const TrapContext, user_token: usize*/) -> ! {
    // see `__snap_trap` for the stack layout
    asm!(
        // Return to user space(but still in Supervisor mode)
        "csrw satp, a1",
        "sfence.vma",
        // Make sp -> Trap Context in user stack in user space
        "csrw sscratch, a0",
        "mv sp, a0",
        // Restore privileged registers
        // sstatus
        "ld t0, 256(sp)",
        "csrw sstatus, t0",
        // sepc
        "ld t0, 264(sp)",
        "csrw sepc, t0",
        // Restore all registers
        // Ignore x0
        "ld ra, 8(sp)", // x1
        // Skip sp(x2) as we need it to restore the stack
        "ld gp, 24(sp)",   // x3
        "ld tp, 32(sp)",   // x4
        "ld t0, 40(sp)",   // x5
        "ld t1, 48(sp)",   // x6
        "ld t2, 56(sp)",   // x7
        "ld s0, 64(sp)",   // x8 aka. fp
        "ld s1, 72(sp)",   // x9
        "ld a0, 80(sp)",   // x10 param0/ret
        "ld a1, 88(sp)",   // x11 param1
        "ld a2, 96(sp)",   // x12
        "ld a3, 104(sp)",  // x13
        "ld a4, 112(sp)",  // x14
        "ld a5, 120(sp)",  // x15
        "ld a6, 128(sp)",  // x16
        "ld a7, 136(sp)",  // x17
        "ld s2, 144(sp)",  // x18
        "ld s3, 152(sp)",  // x19
        "ld s4, 160(sp)",  // x20
        "ld s5, 168(sp)",  // x21
        "ld s6, 176(sp)",  // x22
        "ld s7, 184(sp)",  // x23
        "ld s8, 192(sp)",  // x24
        "ld s9, 200(sp)",  // x25
        "ld s10, 208(sp)", // x26
        "ld s11, 216(sp)", // x27
        "ld t3, 224(sp)",  // x28
        "ld t4, 232(sp)",  // x29
        "ld t5, 240(sp)",  // x30
        "ld t6, 248(sp)",  // x31
        // Restore sp
        "ld sp, 16(sp)", // x2
        // Return to user mode
        "sret",
        options(noreturn)
    );
}
