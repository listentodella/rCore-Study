#![no_std]
#![no_main]

use core::arch::global_asm;

mod console;
mod lang_item;
mod uart;

global_asm!(include_str!("boot.S"));

#[no_mangle]
fn kernel_main() -> ! {
    let a = 12u8;
    let b = 32u8;
    let c = a + b;
    uart::init();
    uart::putchar('h' as usize);
    uart::putchar('e' as usize);
    uart::putchar('l' as usize);
    uart::putchar('l' as usize);
    uart::putchar('o' as usize);

    println!("hello myOS, {}", c);
    loop {}
}
