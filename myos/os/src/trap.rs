use core::arch::global_asm;
use riscv::{
    interrupt::supervisor::{Exception, Interrupt},
    register::{
        scause::{Scause, Trap},
        sie, sscratch,
        stvec::{self, TrapMode},
    },
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
fn do_exception(regs: &mut KernelTrapRegs, scause: usize) {
    println!("scause = 0x{:016x}", scause);
    // 其实可以直接用 scause::read() 来获取
    // 这里指示展示寄存器传参,以及 transmute 的简单使用
    let scause: Scause = unsafe { core::mem::transmute(scause) };
    // 新版本的riscv crate 废弃了直接的枚举, 是因为riscv是开放的, 允许扩展新的异常类型
    let raw_trap = scause.cause();
    let trap: Trap<Interrupt, Exception> = raw_trap.try_into().unwrap();
    match trap {
        Trap::Interrupt(interrupt) => match interrupt {
            Interrupt::SupervisorTimer => {
                println!("SupervisorTimer handle");
            }
            Interrupt::SupervisorSoft => {
                println!("SupervisorSoft handle");
            }
            Interrupt::SupervisorExternal => {
                println!("SupervisorExternal handle");
            }
        },
        Trap::Exception(exception) => match exception {
            Exception::InstructionMisaligned => {
                println!("InstructionMisaligned handle");
            }
            Exception::InstructionPageFault => {
                println!("InstructionPageFault handle");
            }
            Exception::StoreFault => {
                println!("StoreFault handle");
                trap_error(regs);
            }
            Exception::LoadFault => {
                println!("LoadFault handle");
                trap_error(regs);
            }
            Exception::StorePageFault => {
                println!("StorePageFault handle");
            }
            Exception::LoadPageFault => {
                println!("LoadPageFault handle");
            }
            Exception::StoreMisaligned => {
                println!("StoreMisaligned handle");
            }
            Exception::LoadMisaligned => {
                println!("LoadMisaligned handle");
            }
            Exception::Breakpoint => {
                println!("Breakpoint handle");
            }
            Exception::IllegalInstruction => {
                println!("IllegalInstruction handle");
            }
            _ => {
                println!("unknown exception {:?}", exception);
            }
        },
    }
}

fn trap_error(regs: &mut KernelTrapRegs) {
    println!("{:#x?}", regs);
    panic!();
}

#[repr(C)]
#[derive(Debug)]
struct KernelTrapRegs {
    // spec + 31 common registers
    sepc: usize,
    ra: usize,
    sp: usize,
    gp: usize,
    tp: usize,
    t0: usize,
    t1: usize,
    t2: usize,
    s0: usize,
    s1: usize,
    a0: usize,
    a1: usize,
    a2: usize,
    a3: usize,
    a4: usize,
    a5: usize,
    a6: usize,
    a7: usize,
    s2: usize,
    s3: usize,
    s4: usize,
    s5: usize,
    s6: usize,
    s7: usize,
    s8: usize,
    s9: usize,
    s10: usize,
    s11: usize,
    t3: usize,
    t4: usize,
    t5: usize,
    t6: usize,
    // Supervisor CSRs
    sstatus: usize,
    sbadaddr: usize,
    scause: usize,
    // a0 value before the syscall
    orig_a0: usize,
}
