use core::arch::asm;

pub unsafe fn print_stack_trace() {
    let mut fp: *const usize;
    asm!("mv {}, fp", out(reg) fp);

    let mut saved_ra = *fp.sub(1);
    let mut saved_fp = *fp.sub(2);

    println!("== Begin stack trace ==");
    while !fp.is_null() && saved_fp != 0 {
        saved_ra = *fp.sub(1);
        saved_fp = *fp.sub(2);

        println!("0x{:016x}, fp = 0x{:016x}", saved_ra, saved_fp);

        fp = saved_fp as *const usize;
    }
    println!("== End stack trace ==");
}
