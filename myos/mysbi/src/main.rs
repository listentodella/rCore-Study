#![no_std]
#![no_main]

use core::arch::asm;
use core::arch::global_asm;
use riscv::register::{mepc, mstatus, satp, sie, stvec};

mod console;
mod lang_item;
mod sbi_trap;
mod uart;

global_asm!(include_str!("sbi_boot.S"));

const FW_JUMP_ADDR: usize = 0x8020_0000;

#[no_mangle]
fn sbi_main() {
    uart::init();
    println!(
        r"
 |\   /|     __ __ _
 | \ / |  / (_ |__)|
 |  |  |\/  __)|__)|
--------/-----------
"
    );
    // 设置M模式的异常向量表
    sbi_trap::init();

    // 设置跳转模式为S模式
    let mut val = mstatus::read();
    val.set_mpp(mstatus::MPP::Supervisor);
    val.set_mpie(false);
    mstatus::write(val);

    sbi_trap::delegate();

    // 设置M模式的异常程序计数器, 用于 mret 跳转
    mepc::write(FW_JUMP_ADDR);

    unsafe {
        // 设置S模式的异常向量表入口地址
        //stvec::write(FW_JUMP_ADDR, stvec::TrapMode::Vectored);
        stvec::write(FW_JUMP_ADDR, stvec::TrapMode::Direct);

        // 关闭S模式的中断
        sie::clear_ssoft();
        sie::clear_sext();
        sie::clear_stimer();
    }

    // 关闭S模式的页表转换
    satp::write(0);

    unsafe {
        // 切换到S模式
        asm!("mret");
    }
}
