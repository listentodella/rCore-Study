#[derive(Copy, Clone)]
#[repr(C)] //要与C接口交互,务必要用该属性!
pub struct TaskContext {
    ra: usize,
    sp: usize,
    s: [usize; 12],
}

impl TaskContext {
    /// init task context
    pub fn zero_init() -> Self {
        Self {
            ra: 0,
            sp: 0,
            s: [0; 12],
        }
    }

    pub fn goto_restore(kstack_ptr: usize) -> Self {
        extern "C" {
            fn __restore();
        }
        Self {
            ra: __restore as _,
            sp: kstack_ptr,
            s: [0; 12],
        }
    }
}
