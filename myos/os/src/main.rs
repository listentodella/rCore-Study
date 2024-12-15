#![no_std]
#![no_main]

use core::arch::asm;
use core::arch::global_asm;

mod console;
mod lang_item;
mod uart;

global_asm!(include_str!("boot.S"));
global_asm!(include_str!("asm_test.S"));

unsafe fn load_store_embedded() {
    asm!(
        "
        li t0, 0x80200000

        lb t1, (t0)
        lb t1, 4(t0)
        lb t1, -4(t0)
        ld t1, (t0)
        lb t1, 4(t0)

        lui t0, 0x80200
        lui t1, 0x40200

        ret
"
    );
}

#[no_mangle]
unsafe fn load_store_single_step() {
    //     asm!(
    //         "
    // .align 3
    // .globl my_test_data
    // my_test_data:
    // 	.dword 0x12345678abcdabcd
    // "
    //     );
    //let my_test_data = 0x12345678abcdabcdu64;

    asm!("li t0, 0x80200000");
    asm!("lb t1, (t0)   ");
    asm!("lb t1, 4(t0)  ");
    asm!("lb t1, -4(t0) ");
    asm!("ld t1, (t0)   ");
    asm!("lb t1, 4(t0)  ");
    asm!("lui t0, 0x80200");
    asm!("lui t1, 0x40200");
    asm!("la  t0, my_test_data");
    asm!("lla t1, my_test_data");
    asm!("ret                 ");
}

#[no_mangle]
fn kernel_main() -> ! {
    extern "C" {
        fn load_store_test();
    }
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
        load_store_test();
        load_store_embedded();
        load_store_single_step();
    }

    loop {}
}
