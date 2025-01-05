use crate::syscall::sbi_set_timer;
use riscv::register::sie;

const VIRT_CLINT_ADDR: usize = 0x200_0000;
const VIRT_CLINT_TIMER_CMP: usize = VIRT_CLINT_ADDR + 0x4000;
const VIRT_CLINT_TIMER_VAL: usize = VIRT_CLINT_ADDR + 0xbff8;
const CLINT_TIMER_BASE_FREQ: usize = 0x1000_0000;

fn get_cycles() -> usize {
    unsafe { (VIRT_CLINT_TIMER_VAL as *const usize).read_volatile() }
}

pub fn init() {
    sbi_set_timer(get_cycles() + CLINT_TIMER_BASE_FREQ / 1000);
    unsafe {
        sie::set_stimer();
    }
}
