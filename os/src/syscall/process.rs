use log::*;

// App management syscalls
use crate::batch::run_next_app;

/// task exits and submit an exit code
pub fn sys_exit(exit_code: i32) -> isize {
    info!("[kernel] Application exited with code {}", exit_code);
    run_next_app();
    0
}
