use core::{arch::global_asm, panic};
use riscv::{
    interrupt::machine::{Exception, Interrupt},
    register::{
        mcause::{self, Trap},
        medeleg, mepc, mideleg, mie, mip, mstatus, mtval, mtvec,
    },
    ExceptionNumber,
};

use crate::{print, println, uart};

global_asm!(include_str!("sbi_entry.S"));

pub fn init() {
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

pub fn delegate() {
    unsafe {
        // 将如下中断委托给S mode
        mideleg::set_stimer();
        mideleg::set_ssoft();
        mideleg::set_sext();

        // 将如下异常委托给S mode
        // 指令地址未对齐异常
        medeleg::set_instruction_misaligned();
        // 指令缺页异常
        medeleg::set_instruction_page_fault();
        // 加载缺页异常
        medeleg::set_load_page_fault();
        // 存储/AMO缺页异常
        medeleg::set_store_page_fault();
        // 断点异常
        medeleg::set_breakpoint();
        // 系统调用异常
        medeleg::set_user_env_call();
        // 存储访问异常
        medeleg::set_store_fault();
        // 加载访问异常
        medeleg::set_load_fault();
    }
}

#[repr(C)]
struct SbiTrapRegs {
    mepc: usize,
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
    mstatus: usize,
}

#[no_mangle]
//pub fn sbi_trap_handler(regs:&mut SbiTrapRegs) {
// 是否有必要返回可变引用呢?
pub fn sbi_trap_handler(regs: &mut SbiTrapRegs) -> &mut SbiTrapRegs {
    let mut ret = 233;
    let mcause = mcause::read();
    let ecall_id = regs.a7;
    let mut msg = "which handle ???";
    let raw_trap = mcause.cause();
    let trap: Trap<Interrupt, Exception> = raw_trap.try_into().unwrap();
    match trap {
        Trap::Exception(exception) => {
            if exception.number() == 0x09 {
                //println!("sbi syscalled {}", ecall_id);
                ret = sbi_ecall_handle(ecall_id, regs);
                msg = "ecall handle failed!";
            } else {
                println!("which exception {:?}", exception);
            }
        }
        Trap::Interrupt(interrupt) => match interrupt {
            Interrupt::MachineTimer => {
                //println!("interrupt triggered {:?}", interrupt);
                sbi_timer_process();
                ret = 0;
            }
            _ => {
                ret = 1;
                msg = "unsupported interrupt";
            }
        },
    }

    if ret != 0 {
        sbi_trap_error(regs, msg, ret);
    }
    regs
}

const SBI_SET_TIMER: usize = 0;
const SBI_CONSOLE_PUTCHAR: usize = 1;
const SBI_CONSOLE_GETCHAR: usize = 2;
fn sbi_ecall_handle(id: usize, regs: &mut SbiTrapRegs) -> usize {
    let ret = match id {
        SBI_SET_TIMER => {
            clint_timer_event_start(regs.a0);
            0
        }
        SBI_CONSOLE_PUTCHAR => {
            uart::putchar(regs.a0);
            0
        }
        SBI_CONSOLE_GETCHAR => 0,
        _ => 1,
    };

    /* 系统调用返回的是系统调用指令
    （例如ECALL指令）的下一条指令 */
    if ret == 0 {
        regs.mepc += 4;
    }

    ret
}

const VIRT_CLINT_ADDR: usize = 0x200_0000;
const VIRT_CLINT_TIMER_CMP: usize = VIRT_CLINT_ADDR + 0x4000;
const VIRT_CLINT_TIMER_VAL: usize = VIRT_CLINT_ADDR + 0xbff8;
const CLINT_TIMER_BASE_FREQ: usize = 0x1000_0000;
fn clint_timer_event_start(next_event: usize) {
    unsafe {
        (VIRT_CLINT_TIMER_CMP as *mut usize).write_volatile(next_event);
        // 清S模式的timer pending irq
        mip::clear_stimer();
        // 使能M模式的timer中断
        mie::set_mtimer();
    }
}

fn sbi_timer_process() {
    unsafe {
        // 关闭M模式的timer 中断
        mie::clear_mtimer();
        // 使能S模式的timer pending 中断
        mip::set_stimer();
    }
}

fn sbi_trap_error(regs: &mut SbiTrapRegs, msg: &str, ret: usize) {
    println!("error msg {}, ret = {}", msg, ret);
    // println!(
    //     "mcause: 0x{:x}  mtval: 0x{:x} ",
    //     mcause::read().cause(),
    //     mtval::read()
    // );
    println!(
        "mepc: 0x{:016x} mstatus : 0x{:016x}",
        regs.mepc, regs.mstatus
    );
    println!(
        " gp : 0x{:016x}, tp : 0x{:016x}, t0 : 0x{:016x}",
        regs.gp, regs.tp, regs.t0
    );
    println!(
        " t1 : 0x{:016x}, t2 : 0x{:016x}, t3 : 0x{:016x}",
        regs.t1, regs.t2, regs.s0
    );
    println!(
        " s1 : 0x{:016x}, a0 : 0x{:016x}, a1 : 0x{:016x}",
        regs.s1, regs.a0, regs.a1
    );
    println!(
        " a2 : 0x{:016x}, a3 : 0x{:016x}, a4 : 0x{:016x}",
        regs.a2, regs.a3, regs.a4
    );
    println!(
        " a5 : 0x{:016x}, a6 : 0x{:016x}, a7 : 0x{:016x}",
        regs.a5, regs.a6, regs.a7
    );
    println!(
        " s2 : 0x{:016x}, s3 : 0x{:016x}, s4 : 0x{:016x}",
        regs.s2, regs.s3, regs.s4
    );
    println!(
        " s5 : 0x{:016x}, s6 : 0x{:016x}, s7 : 0x{:016x}",
        regs.s5, regs.s6, regs.s7
    );
    println!(
        " s8 : 0x{:016x}, s9 : 0x{:016x}, s10: 0x{:016x}",
        regs.s8, regs.s9, regs.s10
    );
    println!(
        " s11: 0x{:016x}, t3 : 0x{:016x}, t4 : 0x{:016x}",
        regs.s11, regs.t3, regs.t4
    );
    println!(
        " t5 : 0x{:016x}, t6 : 0x{:016x}, sp : 0x{:016x}",
        regs.t5, regs.t6, regs.sp
    );
    println!(" ra : 0x{:016x}", regs.ra);

    panic!("!!!SBI PANIC!!!")
}
