use super::TaskContext;
use crate::syscall::SyscallID;

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum TaskStatus {
    Uninit,
    Ready,
    Running,
    Exited,
}

#[derive(Copy, Clone)]
pub struct TaskControlBlock {
    //pub task_status: TaskStatus,
    pub task_ctx: TaskContext,
    pub task_info: TaskInfo,
}

#[derive(Copy, Clone, Debug)]
pub struct TaskInfo {
    // task id
    //pub id: usize,
    pub status: TaskStatus,
    pub syscall: [SyscallInfo; core::mem::variant_count::<SyscallID>()],
    //pub time: usize,
    pub user_time: usize,
    pub kernel_time: usize,
}

#[derive(Copy, Clone, Debug)]
pub struct SyscallInfo {
    // syscall id
    pub id: SyscallID,
    // 这里是统计系统调用触发的次数, 而不是运行时间
    pub times: usize,
}

impl TaskInfo {
    pub fn init() -> Self {
        Self {
            //id: 0,
            status: TaskStatus::Uninit,
            syscall: [SyscallInfo {
                id: SyscallID::Invalid,
                times: 0,
            }; core::mem::variant_count::<SyscallID>()],
            //time: 0,
            user_time: 0,
            kernel_time: 0,
        }
    }
}
