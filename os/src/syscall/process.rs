use log::*;

// App management syscalls
use crate::batch::run_next_app;
use crate::timer::get_time_us;

/// task exits and submit an exit code
pub fn sys_exit(exit_code: i32) -> ! {
    info!("[kernel] Application exited with code {}", exit_code);
    run_next_app()
}

pub fn sys_get_time(ts: *mut TimeVal, _tz: usize) -> isize {
    let timestamp = get_time_us();
    let sec = timestamp / (1_000_000);
    let usec = timestamp - sec * 1_000_000;
    unsafe {
        ts.write_volatile(TimeVal { sec, usec });
    }
    0
}

#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}
