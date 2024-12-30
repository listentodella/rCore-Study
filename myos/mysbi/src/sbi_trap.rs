use core::arch::global_asm;

global_asm!(include_str!("sbi_entry.S"));
