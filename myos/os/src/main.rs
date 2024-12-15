#![no_std]
#![no_main]

use core::arch::asm;
use core::arch::global_asm;

mod console;
mod lang_item;
mod uart;

mod base;

global_asm!(include_str!("boot.S"));

#[no_mangle]
fn kernel_main() -> ! {
    let a = 12u8;
    let b = 32u8;
    let c = a + b;
    uart::init();
    uart::putchar('r' as usize);
    uart::putchar('i' as usize);
    uart::putchar('s' as usize);
    uart::putchar('c' as usize);
    uart::putchar('v' as usize);

    println!(" hello myOS, {}", c);

    unsafe {
        base::load_store::global_asm_test();
        base::load_store::asm_all();
        base::load_store::asm_single();
        base::pc::rel();
    }

    loop {}
}
