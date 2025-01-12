mod fs;
use fs::*;

mod process;
use process::*;

// const SYSCALL_WRITE: usize = 64;
// const SYSCALL_EXIT: usize = 93;
// const SYSCALL_TS: usize = 169;
// const SYSCALL_YIELD: usize = 124;
// const SYSCALL_TASK_INFO: usize = 410;

#[derive(Debug, PartialEq, Copy, Clone)]
#[non_exhaustive]
pub enum SyscallID {
    Invalid = -1,
    Write = 64,
    Exit = 93,
    Ts = 169,
    Yield = 124,
    TaskInfo = 410,
}

impl From<SyscallID> for usize {
    fn from(val: SyscallID) -> Self {
        val as usize
    }
}

impl From<usize> for SyscallID {
    fn from(val: usize) -> Self {
        match val {
            64 => Self::Write,
            93 => Self::Exit,
            169 => Self::Ts,
            124 => Self::Yield,
            410 => Self::TaskInfo,
            _ => Self::Invalid,
        }
    }
}

pub fn syscall(syscall_id: usize, args: [usize; 3]) -> isize {
    match syscall_id.into() {
        SyscallID::Write => sys_write(args[0], args[1] as *const u8, args[2]),
        SyscallID::Exit => sys_exit(args[0] as i32),
        SyscallID::Ts => sys_get_time(args[0] as *mut TimeVal, 0),
        SyscallID::Yield => sys_yield(),
        //SYSCALL_TASK_INFO => sys_task_info(args[0], args[1] as *mut TaskInfo),
        _ => panic!("Unsupported syscall_id: {}", syscall_id),
    }
}
