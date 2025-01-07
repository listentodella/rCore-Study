#![no_std]
// 启用linkage功能
#![feature(linkage)]

// 用于在使用宏时指示编译器引入宏定义
// TODO: 这与宏的作用域有关
#[macro_use]
pub mod console;
mod lang_items;
mod syscall;

// linkage宏将该函数放在 .text.entry 代码段中
// 方便在后续链接的时候调整它的位置,使得它能够作为用户库的入口
#[no_mangle]
#[link_section = ".text.entry"]
pub extern "C" fn _start() -> ! {
    clear_bss();
    exit(main());
    panic!("unreachable after sys_exit!");
}

fn clear_bss() {
    extern "C" {
        fn start_bss();
        fn end_bss();
    }

    (start_bss as usize..end_bss as usize).for_each(|addr| unsafe {
        (addr as *mut u8).write_volatile(0);
    });
}

// 该宏将被修饰的函数标识为弱链接
// 如果在最终链接时, 其他地方若有同名函数, 则会链接另一个函数
#[linkage = "weak"]
#[no_mangle]
fn main() -> i32 {
    panic!("Cannot find main!");
}

use syscall::*;
pub fn write(fd: usize, buffer: &[u8]) -> isize {
    sys_write(fd, buffer)
}

pub fn exit(exit_code: i32) -> isize {
    sys_exit(exit_code)
}

// yield 是 rust 的关键字, 所以只能改名
pub fn yield_() -> isize {
    sys_yield()
}
