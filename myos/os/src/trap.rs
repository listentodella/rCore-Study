use core::arch::global_asm;
use riscv::register::{
    sie, sscratch,
    stvec::{self, TrapMode},
};

use crate::println;

global_asm!(include_str!("entry.S"));

pub fn init() {
    extern "C" {
        fn do_exception_vector();
    }

    sscratch::write(0);

    unsafe {
        // set exception vector for S mode
        stvec::write(do_exception_vector as usize, TrapMode::Direct);
        println!(
            "stvec = {:?}, exception vector addr = {}",
            stvec::read(),
            do_exception_vector as usize
        );
        // enable all interrups
        sie::set_sext();
        sie::set_ssoft();
        sie::set_stimer();
    }
}

#[no_mangle]
fn do_exception() {}
