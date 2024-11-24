#![no_std]
#![no_main]

mod lang_items;

use core::arch::global_asm;

// include_str! 宏, 可以将指令路径下的文件转化为字符串
// 再通过global_asm!宏嵌入到代码中
global_asm!(include_str!("entry.asm"));
