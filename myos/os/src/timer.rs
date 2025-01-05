use crate::{println, syscall::sbi_set_timer};
use core::sync::atomic::{AtomicUsize, Ordering};
use riscv::register::sie;

const VIRT_CLINT_ADDR: usize = 0x200_0000;
const VIRT_CLINT_TIMER_CMP: usize = VIRT_CLINT_ADDR + 0x4000;
const VIRT_CLINT_TIMER_VAL: usize = VIRT_CLINT_ADDR + 0xbff8;
const CLINT_TIMER_BASE_FREQ: usize = 0x1000_0000;

static JIFFIES: AtomicUsize = AtomicUsize::new(0);

fn get_cycles() -> usize {
    unsafe { (VIRT_CLINT_TIMER_VAL as *const usize).read_volatile() }
}

pub fn init() {
    sbi_set_timer(get_cycles() + CLINT_TIMER_BASE_FREQ / 1000);
    unsafe {
        sie::set_stimer();
    }
}

pub fn handle_timer_irq() {
    unsafe {
        sie::clear_stimer();
    }
    //init();
    let val = JIFFIES.load(Ordering::Relaxed);
    JIFFIES.store(val.wrapping_add(1), Ordering::Relaxed);
    println!("Core0 Timer jiffies = {:?}", JIFFIES);
}
