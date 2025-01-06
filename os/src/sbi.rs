pub fn console_putchar(c: usize) {
    #[allow(deprecated)]
    sbi_rt::legacy::console_putchar(c);
}

pub fn shutdown(failure: bool) -> ! {
    use sbi_rt::{system_reset, NoReason, Shutdown, SystemFailure};
    if !failure {
        system_reset(Shutdown, NoReason);
    } else {
        system_reset(Shutdown, SystemFailure);
    }

    unreachable!()
}

pub fn set_timer(timer: usize) {
    //const SBI_SET_TIMER: usize = 0;
    //sbi_rt::sbi_call(SBI_SET_TIMER, timer, 0, 0);
    sbi_rt::set_timer(timer as _);
}
