use crate::syscall::syscall;
use crate::task::{exit_current_and_run_next, suspend_current_and_run_next, trace_syscall_info};
//use crate::{batch::run_next_app, timer::set_next_trigger};
use crate::timer::set_next_trigger;
use core::arch::global_asm;
use riscv::register::{
    mtvec::TrapMode,
    scause::{self, Exception, Interrupt, Trap},
    stval, stvec,
};
use riscv::register::{sie, sstatus};

use log::*;

mod context;
pub use context::TrapContext;

global_asm!(include_str!("trap.S"));

// 一般不去使用 clear_sie/set_sie来开关中断
// 因为riscv在中断触发时,自动关闭sstatus.SIE,sret返回时自动打开sstatus.SIE
static mut KERNEL_INTERRUPT_TRIGGERED: bool = false;

pub fn check_kernel_interrupt() -> bool {
    unsafe { (&raw mut KERNEL_INTERRUPT_TRIGGERED).read_volatile() }
    //unsafe { KERNEL_INTERRUPT_TRIGGERED }
    //false
}

pub fn mark_kernel_interrupt() {
    unsafe { (&raw mut KERNEL_INTERRUPT_TRIGGERED).write_volatile(true) }
}

/// initialize CSR `stvec` as the entry of `__alltraps`
pub fn init() {
    extern "C" {
        fn __alltraps();
    }
    unsafe {
        stvec::write(__alltraps as usize, TrapMode::Direct);
    }
}

pub fn enable_timer_interrupt() {
    debug!("[kernel] enable timer interrupt");
    unsafe {
        sie::set_stimer();
        //sstatus::set_sie();
    }
}

#[no_mangle]
pub fn trap_handler(ctx: &mut TrapContext) -> &mut TrapContext {
    match sstatus::read().spp() {
        sstatus::SPP::Supervisor => kernel_trap_handler(ctx),
        sstatus::SPP::User => user_trap_handler(ctx),
    }
}

fn kernel_trap_handler(ctx: &mut TrapContext) -> &mut TrapContext {
    let scause = scause::read();
    let stval = stval::read();
    match scause.cause() {
        Trap::Interrupt(Interrupt::SupervisorTimer) => {
            // if use trace, will cause high-freq unexpected exception!!!
            // 如果sscratch初始值使用entry.asm那种方式规避的话,就会遇到问题
            // 使用trap.S里的方案则可彻底规避
            trace!("[kenrel] interrupt: from timer");
            // println is ok...
            //println!("[kenrel] interrupt: from timer");
            mark_kernel_interrupt();
            set_next_trigger();
            // 这是kernel自己的异常,暂不涉及调度
        }
        Trap::Exception(Exception::StoreFault) | Trap::Exception(Exception::StorePageFault) => {
            panic!("[kernel] PageFault in kernel, bad addr = {:#x}, bad instruction = {:#x}, ctx = {:#x?}, kernel killed it",
            stval, ctx.sepc, ctx);
            // 这是kernel自己的异常,暂不涉及调度
            // exit_current_and_run_next();
        }
        _ => {
            error!("[kernel] unknown exception or interrupt");
            // 这是kernel自己的异常,暂不涉及调度
            //exit_current_and_run_next();
        }
    }

    ctx
}

fn user_trap_handler(ctx: &mut TrapContext) -> &mut TrapContext {
    // 从此开始属于kernel, 也是user time的暂停/停止点
    crate::task::user_end_and_kernel_time_start();
    let scause = scause::read();
    let stval = stval::read();
    match scause.cause() {
        Trap::Exception(Exception::UserEnvCall) => {
            ctx.sepc += 4;
            trace_syscall_info(ctx.x[17]);
            ctx.x[10] = syscall(ctx.x[17], [ctx.x[10], ctx.x[11], ctx.x[12]]) as usize;
        }
        Trap::Exception(Exception::StoreFault) | Trap::Exception(Exception::StorePageFault) => {
            error!("[kernel] PageFault in application, kernel killed it.");
            //run_next_app();
            exit_current_and_run_next();
        }
        Trap::Exception(Exception::IllegalInstruction) => {
            error!("[kernel] IllegalInstruction in application, kernel killed it.");
            //run_next_app();
            exit_current_and_run_next();
        }
        Trap::Interrupt(Interrupt::SupervisorTimer) => {
            debug!("[kernel] SupervisorTimer");
            set_next_trigger();
            suspend_current_and_run_next();
        }
        _ => {
            panic!(
                "Unsupported trap {:?}, stval = {:#x}!",
                scause.cause(),
                stval
            );
        }
    }
    // 从此开始属于user, 也是kernel time 的暂停/停止点
    crate::task::kernel_end_and_user_time_start();
    ctx
}
