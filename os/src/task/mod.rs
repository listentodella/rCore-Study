mod context;
use crate::{config::MAX_APP_NUM, sbi::shutdown, syscall::SyscallID, timer::get_time_us};
use context::TaskContext;
use core::panic;
use lazy_static::lazy_static;
use switch::__switch;
use task::{TaskControlBlock, TaskInfo, TaskStatus};

use crate::{
    loader::{get_num_app, init_app_ctx},
    sync::UPSafeCell,
};
use log::{info, trace};
mod switch;

// 该属性可以避免clippy的warning
#[allow(clippy::module_inception)]
mod task;

pub struct TaskManager {
    num_app: usize,
    inner: UPSafeCell<TaskManagerInner>,
}

struct TaskManagerInner {
    tasks: [TaskControlBlock; MAX_APP_NUM],
    current_task: usize,
    last_ts: usize,
}

impl TaskManagerInner {
    // 每次会返回当前到上一次暂停的时间间隔
    // 然后刷新为当前时间
    fn update_duration(&mut self) -> usize {
        let tmp_ts = self.last_ts;
        self.last_ts = get_time_us();
        self.last_ts - tmp_ts
    }
}

lazy_static! {
    pub static ref TASK_MANAGER: TaskManager = {
        let num_app = get_num_app();
        let mut tasks = [TaskControlBlock {
            task_ctx: TaskContext::zero_init(),
            task_info: TaskInfo::init()
        }; MAX_APP_NUM];
        // app第一次被运行之前, 在这里构造任务上下文
        // 即通过app_init_ctx 将app的入口地址, sp 压入栈顶
        // 然后通过goto_restore构造TaskControlBlock要用到的TaskContext
        // for i in 0..num_app {
        //     tasks[i].task_ctx = TaskContext::goto_restore(init_app_ctx(i));
        //     tasks[i].task_info.status = TaskStatus::Ready;
        // }
        for (i, task) in tasks.iter_mut().enumerate() {
            task.task_ctx = TaskContext::goto_restore(init_app_ctx(i));
            task.task_info.status = TaskStatus::Ready;
        }
        TaskManager {
            num_app,
            inner: unsafe {
                UPSafeCell::new(TaskManagerInner {
                    tasks,
                    current_task: 0,
                    last_ts:0
                })
            },
        }
    };
}

impl TaskManager {
    fn trace_syscall_info(&self, syscall_id: usize) {
        let mut inner = self.inner.exclusive_access();
        let current = inner.current_task;
        let syscall_id: SyscallID = syscall_id.into();

        let idx = match syscall_id {
            SyscallID::Write => 0,
            SyscallID::Exit => 1,
            SyscallID::Yield => 2,
            SyscallID::Ts => 3,
            //SYSCALL_TASK_INFO => sys_task_info(args[0], args[1] as *mut TaskInfo),
            _ => 5,
        };
        inner.tasks[current].task_info.syscall[idx].times += 1;
        inner.tasks[current].task_info.syscall[idx].id = syscall_id;
    }
    fn mark_current_suspended(&self) {
        let mut inner = self.inner.exclusive_access();
        let current = inner.current_task;
        // 当被标记为suspend,即Ready时, 意味着该app不再占用kernel time了
        inner.tasks[current].task_info.kernel_time += inner.update_duration();
        trace!("task {} suspended", current);
        inner.tasks[current].task_info.status = TaskStatus::Ready;
    }
    fn mark_current_exited(&self) {
        let mut inner = self.inner.exclusive_access();
        let current = inner.current_task;
        // 当被标记为exit时, 意味着该app不再占用kernel time了
        inner.tasks[current].task_info.kernel_time += inner.update_duration();

        trace!(
            "task {} syscall trace {:?}",
            current,
            inner.tasks[current].task_info
        );
        inner.tasks[current].task_info.status = TaskStatus::Exited;
    }

    fn find_next_task(&self) -> Option<usize> {
        let inner = self.inner.exclusive_access();
        let current = inner.current_task;
        (current + 1..current + self.num_app + 1)
            .map(|id| id % self.num_app)
            .find(|id| inner.tasks[*id].task_info.status == TaskStatus::Ready)
    }

    fn run_next_task(&self) {
        if let Some(next) = self.find_next_task() {
            let mut inner = self.inner.exclusive_access();
            let current = inner.current_task;
            trace!("task {} running", current);
            inner.tasks[next].task_info.status = TaskStatus::Running;
            inner.current_task = next;
            let current_task_ctx_ptr = &mut inner.tasks[current].task_ctx as *mut TaskContext;
            let next_task_ctx_ptr = &inner.tasks[next].task_ctx as *const TaskContext;
            drop(inner);
            // 必须在该代码块之前手动drop,因为一时半会回不来了
            // 直到下次切换到该app时,才算"回来"
            // 在此期间,TaskManager的exclusive_access永远成功不了
            unsafe {
                __switch(current_task_ctx_ptr, next_task_ctx_ptr);
            }
            // go back to user mode
        } else {
            //panic!("[kernel] all apps completed!");
            info!("[kernel] all apps completed!");
            //use crate::board::QEMUExit;
            //crate::board::QEMU_EXIT_HANDLE.exit_success();
            shutdown(true);
        }
    }

    fn run_first_task(&self) -> ! {
        let mut inner = self.inner.exclusive_access();
        let task0 = &mut inner.tasks[0];
        task0.task_info.status = TaskStatus::Running;
        let next_task_ctx_ptr = &task0.task_ctx as *const TaskContext;
        // 开始记录时间
        inner.update_duration();
        drop(inner);
        let mut _unused = TaskContext::zero_init();
        unsafe {
            __switch(&mut _unused as *mut TaskContext, next_task_ctx_ptr);
        }

        panic!("unreachable in run_first_task!");
    }

    fn kernel_end_and_user_time_start(&self) {
        let mut inner = self.inner.exclusive_access();
        let current = inner.current_task;
        inner.tasks[current].task_info.kernel_time += inner.update_duration();
    }

    fn user_end_and_kernel_time_start(&self) {
        let mut inner = self.inner.exclusive_access();
        let current = inner.current_task;
        inner.tasks[current].task_info.user_time += inner.update_duration();
    }
}

fn mark_current_suspended() {
    TASK_MANAGER.mark_current_suspended();
}

fn mark_current_exited() {
    TASK_MANAGER.mark_current_exited();
}

fn run_next_task() {
    TASK_MANAGER.run_next_task();
}

pub fn suspend_current_and_run_next() {
    mark_current_suspended();
    run_next_task();
}

pub fn exit_current_and_run_next() {
    mark_current_exited();
    run_next_task();
}

pub fn run_first_task() -> ! {
    TASK_MANAGER.run_first_task()
}

pub fn trace_syscall_info(syscall_id: usize) {
    TASK_MANAGER.trace_syscall_info(syscall_id);
}

/*
 * 时间统计规则
 * 1. 当 first_task 执行后,第一个app肯定是运行在user, 记为 t0
 * 2. 之后, app与kernel切换的时机,一个是异常,一个是系统调用
 *   2.1 当 switch 执行时, cpu/app 便从kernel转为user
 *   2.2 当异常或系统调用触发时,cpu/app 便从user转为kernel
 * 因此, 当第一个异常或系统调用触发时, 运行到 trap_handler 时, 可以第一时间记录为 t1
 * t1 - t0 即为第一个app运行 user 的持续时间
 * 之后切换到下一个app时, 即t2, 仍属于第一个app的 kernel 占用时间
 * apps的kernel+user time 便如此反复
 */

pub fn kernel_end_and_user_time_start() {
    TASK_MANAGER.kernel_end_and_user_time_start();
}

pub fn user_end_and_kernel_time_start() {
    TASK_MANAGER.user_end_and_kernel_time_start();
}
