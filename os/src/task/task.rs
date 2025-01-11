use super::TaskContext;

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

const MAX_SYSCALL_NUM: usize = 5;
#[derive(Copy, Clone, Debug)]
pub struct TaskInfo {
    // task id
    pub id: usize,
    pub status: TaskStatus,
    pub syscall: [SyscallInfo; MAX_SYSCALL_NUM],
    pub time: usize,
}

#[derive(Copy, Clone, Debug)]
pub struct SyscallInfo {
    // syscall id
    pub id: usize,
    // 这里是统计系统调用触发的次数, 而不是运行时间
    pub times: usize,
}

impl TaskInfo {
    pub fn init() -> Self {
        Self {
            id: 0,
            status: TaskStatus::Uninit,
            syscall: [SyscallInfo { id: 0, times: 0 }; MAX_SYSCALL_NUM],
            time: 0,
        }
    }
}
