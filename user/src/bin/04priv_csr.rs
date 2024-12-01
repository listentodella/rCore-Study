#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

use riscv::register::sstatus::{self, SPP};

#[no_mangle]
fn main() -> i32 {
    println!("Try to access privileged CSR in U mode");
    println!("Kernel should kill this app!");
    unsafe {
        sstatus::set_spp(SPP::User);
    }

    0
}
