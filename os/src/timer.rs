use crate::config::CLOCK_FREQ;
use crate::sbi;
use riscv::register::time;

pub fn get_time() -> usize {
    time::read()
}

const TICKS_PER_SEC: usize = 100;

pub fn set_next_trigger() {
    sbi::set_timer(get_time() + CLOCK_FREQ / TICKS_PER_SEC);
}

const MICRO_PER_SEC: usize = 1_000_000;

pub fn get_time_us() -> usize {
    time::read() / (CLOCK_FREQ / MICRO_PER_SEC)
}
