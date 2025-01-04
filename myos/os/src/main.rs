#![no_std]
#![no_main]
#![feature(naked_functions)]
use core::arch::asm;
use core::arch::global_asm;

mod console;
mod lang_item;
mod syscall;
mod trap;
mod uart;

mod base;

global_asm!(include_str!("boot.S"));

unsafe fn base_asm_test() {
    base::load_store::global_asm_test();
    base::load_store::asm_all();
    base::load_store::asm_single();
    base::pc::rel();
    base::shift::test();
    base::add_sub::test();
    let a = 100u64;
    let b = 1000u64;
    if base::compare::is_little_than(a, b) {
        //println!("{} < {}", a, b);
        println!("<");
    } else {
        //println!("{} >= {}", a, b);
        println!("!<");
    }

    let a = 100u64;
    let b = 1000u64;
    if base::compare::is_little_than(b, a) {
        //println!("{} < {}", a, b);
        println!("<");
    } else {
        println!("!<");
    }

    base::compare::naked_is_little_than();

    if base::compare::is_zero(0) {
        println!("get zero!");
    }
    if !base::compare::is_zero(2) {
        println!("get non-zero!");
    }
    //base::fp::print_backtrace();

    //panic!();

    base::load_store::memcpy(0x80200000u64, 0x80800000u64, 32u64);

    base::load_store::memset(0x8080_0000, 0xFF, 32);

    base::csr::csrrw();
}

#[no_mangle]
fn kernel_main() -> ! {
    clear_bss();
    syscall::sbi_put_string("Hello SBI syscall!\n");
    let a = 12u8;
    let b = 32u8;
    let c = a + b;
    uart::init();
    uart::putchar('r' as usize);
    uart::putchar('i' as usize);
    uart::putchar('s' as usize);
    uart::putchar('c' as usize);
    uart::putchar('v' as usize);
    trap::init();

    println!("hello myOS, {}", c);

    unsafe {
        base_asm_test();
    }

    loop {}
}

fn clear_bss() {
    extern "C" {
        fn sbss();
        fn ebss();
    }
    (sbss as usize..ebss as usize).for_each(|addr| unsafe {
        (addr as *mut u8).write_volatile(0);
    });
}
