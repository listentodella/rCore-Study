#![no_std]
#![no_main]

use core::arch::global_asm;

mod lang_item;

global_asm!(include_str!("boot.S"));

#[no_mangle]
fn kernel_main() -> ! {
    let a = 12u8;
    let b = 32u8;
    let _c = a + b;

    loop {}
}
