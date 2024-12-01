#![no_std]
#![no_main]

use core::arch::asm;
#[macro_use]
extern crate user_lib;

#[no_mangle]
fn main() -> i32 {
    println!("Try to execute privileged instruction in U mode");
    println!("Kernel should kill this app!");

    unsafe {
        asm!("sret");
    }

    0
}
