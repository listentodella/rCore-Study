use core::arch::global_asm;

global_asm!(include_str!("asm_test.S"));

pub mod add_sub;
pub mod compare;
pub mod csr;
pub mod load_store;
pub mod pc;
pub mod ra;
pub mod shift;
