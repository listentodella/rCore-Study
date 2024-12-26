use core::arch::asm;

use crate::println;

/*
 * 通常情况下,函数的调用栈帧类似一条链表, fp 为节点
 * fp 的地址, 减去1个字节, 即为备份的ra所在的内存地址
 * fp 的地址, 减去2个字节, 即为备份的fp所在的内存地址
 * fp 的内存地址, 则会指向其caller的fp所在的内存地址
 * 如此反复, 直至该链表到头
 * 由于实际应用,链表头的追溯是比较困难的,万一第一个函数入口不够规范,
 * 导致没有结束的标志, 可能永远追不到头?
 */

pub unsafe fn print_backtrace() {
    extern "C" {
        fn kernel_main();
    }

    let mut fp: *const usize = core::ptr::null_mut();
    let mut stop_fp: *const usize = core::ptr::null_mut();
    asm!("mv {}, fp", out(reg) fp);
    asm!("la t0, {}",  sym kernel_main );
    asm!("mv {}, t0",out(reg) stop_fp );
    //stop_fp = 0x80205000 as *const usize;

    while !fp.is_null() {
        //while fp != stop_fp {
        let saved_ra = *fp.sub(1);
        let saved_fp = *fp.sub(2);
        println!("ra = 0x{:016x}, fp = 0x{:016x}", saved_ra, saved_fp);
        fp = saved_fp as *const usize;
    }
}
