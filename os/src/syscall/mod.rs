mod fs;
use fs::*;

mod process;
use process::*;

const SYSCALL_WRITE: usize = 64;
const SYSCALL_EXIT: usize = 93;
const SYSCALL_TS: usize = 169;
const SYSCALL_YIELD: usize = 124;
const SYSCALL_TASK_INFO: usize = 410;

pub fn syscall(syscall_id: usize, args: [usize; 3]) -> isize {
    match syscall_id {
        SYSCALL_WRITE => sys_write(args[0], args[1] as *const u8, args[2]),
        SYSCALL_EXIT => sys_exit(args[0] as i32),
        SYSCALL_TS => sys_get_time(args[0] as *mut TimeVal, 0),
        SYSCALL_YIELD => sys_yield(),
        //SYSCALL_TASK_INFO => sys_task_info(args[0], args[1] as *mut TaskInfo),
        _ => panic!("Unsupported syscall_id: {}", syscall_id),
    }
}
