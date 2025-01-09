mod context;

use crate::{config::MAX_APP_NUM, sbi::shutdown};
use context::TaskContext;
use core::panic;
use lazy_static::lazy_static;
use switch::__switch;
use task::{TaskControlBlock, TaskStatus};

use crate::{
    loader::{get_num_app, init_app_ctx},
    sync::UPSafeCell,
};
use log::info;
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
}

lazy_static! {
    pub static ref TASK_MANAGER: TaskManager = {
        let num_app = get_num_app();
        let mut tasks = [TaskControlBlock {
            task_ctx: TaskContext::zero_init(),
            task_status: TaskStatus::Uninit,
        }; MAX_APP_NUM];
        // app第一次被运行之前, 在这里构造任务上下文
        // 即通过app_init_ctx 将app的入口地址, sp 压入栈顶
        // 然后通过goto_restore构造TaskControlBlock要用到的TaskContext
        // for i in 0..num_app {
        //     tasks[i].task_ctx = TaskContext::goto_restore(init_app_ctx(i));
        //     tasks[i].task_status = TaskStatus::Ready;
        // }
        for (i, task) in tasks.iter_mut().enumerate() {
            task.task_ctx = TaskContext::goto_restore(init_app_ctx(i));
            task.task_status = TaskStatus::Ready;
        }
        TaskManager {
            num_app,
            inner: unsafe {
                UPSafeCell::new(TaskManagerInner {
                    tasks,
                    current_task: 0,
                })
            },
        }
    };
}

impl TaskManager {
    fn mark_current_suspended(&self) {
        let mut inner = self.inner.exclusive_access();
        let current = inner.current_task;
        inner.tasks[current].task_status = TaskStatus::Ready;
    }
    fn mark_current_exited(&self) {
        let mut inner = self.inner.exclusive_access();
        let current = inner.current_task;
        inner.tasks[current].task_status = TaskStatus::Exited;
    }

    fn find_next_task(&self) -> Option<usize> {
        let inner = self.inner.exclusive_access();
        let current = inner.current_task;
        (current + 1..current + self.num_app + 1)
            .map(|id| id % self.num_app)
            .find(|id| inner.tasks[*id].task_status == TaskStatus::Ready)
    }

    fn run_next_task(&self) {
        if let Some(next) = self.find_next_task() {
            let mut inner = self.inner.exclusive_access();
            let current = inner.current_task;
            inner.tasks[next].task_status = TaskStatus::Running;
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
            shutdown(true);
        }
    }

    fn run_first_task(&self) -> ! {
        let mut inner = self.inner.exclusive_access();
        let task0 = &mut inner.tasks[0];
        task0.task_status = TaskStatus::Running;
        let next_task_ctx_ptr = &task0.task_ctx as *const TaskContext;
        drop(inner);
        let mut _unused = TaskContext::zero_init();
        unsafe {
            __switch(&mut _unused as *mut TaskContext, next_task_ctx_ptr);
        }

        panic!("unreachable in run_first_task!");
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
