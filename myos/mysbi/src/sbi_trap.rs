use core::arch::global_asm;
use riscv::register::{mepc, mie, mstatus, mtvec};

global_asm!(include_str!("sbi_entry.S"));

pub fn sbi_trap_init() {
    extern "C" {
        fn sbi_exception_vector();
    }
    unsafe {
        // 设置M模式下异常向量表
        // 直接访问模式:所有陷入M模式的异常或中断,会自动调到BASE字段设置的基地址中
        //              在中断处理函数中再读取mcause,根据触发原因跳转到对应的处理函数
        // 向量访问模式:中断或异常触发后,会跳转到BASE字段指向的异常向量表中,每个向量占4字节
        //              即BASE+4(exception code), 这个excption code是通过查询mcause得到的
        //              e.g. 在M模式下, 时钟中断触发后会跳转到BASE+0x1C地址处
        mtvec::write(sbi_exception_vector as usize, mtvec::TrapMode::Direct);
        // 关闭M模式下所有中断
        mie::clear_msoft();
        mie::clear_mtimer();
        mie::clear_mext();
    }
}
