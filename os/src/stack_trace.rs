use core::arch::asm;

pub unsafe fn print_stack_trace() {
    let mut fp: *const usize = core::ptr::null_mut();
    asm!("mv {}, fp", out(reg) fp);

    println!("=========BACKTRACE START=========");
    while !fp.is_null() {
        let saved_ra = *fp.sub(1);
        let saved_fp = *fp.sub(2);

        println!("ra = 0x{:016x}, fp = 0x{:016x}", saved_ra, saved_fp);

        fp = saved_fp as *const usize;
    }
    println!("=========BACKTRACE END=========");
}
