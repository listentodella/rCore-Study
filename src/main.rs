#![no_std]
#![no_main]

#[macro_use]
mod console;

mod lang_items;
mod sbi;

use core::{arch::global_asm, panic};

// include_str! 宏, 可以将指令路径下的文件转化为字符串
// 再通过global_asm!宏嵌入到代码中
global_asm!(include_str!("entry.asm"));

// 避免编译器对函数名称进行混淆, 否则链接时, entry.asm将找不到该函数
#[no_mangle]
pub fn rust_main() -> ! {
    clear_bss();
    println!("hello rust rcore!");
    // 如果以panic等非正常途径的方式进入发散
    // make 检查返回值会报错, 属于正常现象
    panic!("Manually Shutdown the Machine!");
    //loop {}
}

fn clear_bss() {
    extern "C" {
        fn sbss();
        fn ebss();
    }
    (sbss as usize..ebss as usize).for_each(|a| unsafe { (a as *mut u8).write_volatile(0) });
}
