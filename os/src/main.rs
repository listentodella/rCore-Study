#![no_std]
#![no_main]

#[macro_use]
mod console;

pub mod batch;
mod lang_items;
mod logging;
mod sbi;
mod stack_trace;
mod sync;
mod syscall;
pub mod trap;

use core::arch::global_asm;
use log::*;

// include_str! 宏, 可以将指令路径下的文件转化为字符串
// 再通过global_asm!宏嵌入到代码中
global_asm!(include_str!("entry.asm"));

global_asm!(include_str!("link_app.S"));

// 避免编译器对函数名称进行混淆, 否则链接时, entry.asm将找不到该函数
#[no_mangle]
pub fn rust_main() -> ! {
    extern "C" {
        fn stext(); // begin addr of text segment
        fn etext(); // end addr of text segment
        fn srodata(); // start addr of Read-Only data segment
        fn erodata(); // end addr of Read-Only data ssegment
        fn sdata(); // start addr of data segment
        fn edata(); // end addr of data segment
        fn sbss(); // start addr of BSS segment
        fn ebss(); // end addr of BSS segment
        fn boot_stack_lower_bound(); // stack lower bound
        fn boot_stack_top(); // stack top
        fn _num_app();
    }

    clear_bss();
    println!("[kernel] hello rCore!");

    logging::init();

    trace!(
        "[kernel] .text [{:#x}, {:#x})",
        stext as usize,
        etext as usize
    );
    debug!(
        "[kernel] .rodata [{:#x}, {:#x})",
        srodata as usize, erodata as usize
    );
    info!(
        "[kernel] .data [{:#x}, {:#x})",
        sdata as usize, edata as usize
    );
    warn!(
        "[kernel] boot_stack top=bottom={:#x}, lower_bound={:#x}",
        boot_stack_top as usize, boot_stack_lower_bound as usize
    );
    error!("[kernel] .bss [{:#x}, {:#x})", sbss as usize, ebss as usize);

    trap::init();
    batch::init();
    panic!("Manually Shutdown the Machine!");
    batch::run_next_app();

    // 如果以panic等非正常途径的方式进入发散
    // make 检查返回值会报错, 属于正常现象
    //panic!("Manually Shutdown the Machine!");
    sbi::shutdown(false)
}

fn clear_bss() {
    extern "C" {
        fn sbss();
        fn ebss();
    }
    (sbss as usize..ebss as usize).for_each(|a| unsafe { (a as *mut u8).write_volatile(0) });
}
