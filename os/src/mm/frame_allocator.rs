use super::address::{PhysAddr, PhysPageNum};
use crate::{config::MEMORY_END, sync::UPSafeCell};
use alloc::vec::Vec;
use core::panic;
use lazy_static::lazy_static;

// 描述一个物理页帧管理器需要提供哪些功能
trait FrameAllocator {
    fn new() -> Self;
    // 分配页面
    fn alloc(&mut self) -> Option<PhysPageNum>;
    // 回收页面
    fn dealloc(&mut self, ppn: PhysPageNum);
}

/*
    最简单的栈式物理页帧管理策略
    物理页号区间 [current, end) 此前均未被分配出去过
    recycled向量则以LIFO的方式保存了被回收的物理页号
*/
pub struct StackFrameAllocator {
    current: usize, //空闲内存的起始物理页号
    end: usize,     // 空闲内存的结束物理页号
    recycled: Vec<usize>,
}

impl StackFrameAllocator {
    pub fn init(&mut self, l: PhysPageNum, r: PhysPageNum) {
        self.current = l.0;
        self.end = r.0;
    }
}

impl FrameAllocator for StackFrameAllocator {
    fn new() -> Self {
        Self {
            current: 0,
            end: 0,
            recycled: Vec::new(),
        }
    }

    fn alloc(&mut self) -> Option<PhysPageNum> {
        if let Some(ppn) = self.recycled.pop() {
            Some(ppn.into())
        } else if self.current == self.end {
            None
        } else {
            self.current += 1;
            Some((self.current - 1).into())
        }
    }

    fn dealloc(&mut self, ppn: PhysPageNum) {
        let ppn = ppn.0;
        // validity check
        // 1. 该页面之前一定被分配出去过,因此它的物理页号一定 < current
        // 2. 该页面没有正处于回收状态,因此它不能在recycled找到
        if ppn >= self.current || self.recycled.iter().any(|&v| v == ppn) {
            panic!("Frame ppn={:#x} as not been allocated!", ppn);
        }
        //recycle
        self.recycled.push(ppn);
    }
}

// 使用UPSafeCell封装,确保安全访问
type FrameAllocatorImpl = StackFrameAllocator;
lazy_static! {
    pub static ref FRAME_ALLOCATOR: UPSafeCell<FrameAllocatorImpl> =
        unsafe { UPSafeCell::new(FrameAllocatorImpl::new()) };
}

pub fn init_frame_allocator() {
    extern "C" {
        fn ekernel();
    }
    FRAME_ALLOCATOR.exclusive_access().init(
        PhysAddr::from(ekernel as usize).ceil(),
        PhysAddr::from(MEMORY_END).floor(),
    );
}

pub struct FrameTracker {
    pub ppn: PhysPageNum,
}
impl FrameTracker {
    pub fn new(ppn: PhysPageNum) -> Self {
        //page cleaning
        let bytes_array = ppn.get_bytes_array();
        bytes_array.iter_mut().for_each(|v| *v = 0);
        Self { ppn }
    }
}

impl Drop for FrameTracker {
    fn drop(&mut self) {
        frame_dealloc(self.ppn);
    }
}

// 这里用FrameTracker进一步封装, 是借用了RAII的思想
// 将一个物理页帧的生命周期绑定到一个FrameTracker变量上
// 当它被创建的时候,它一定是被初始化好的,可以安心使用
// 当它的生命周期结束被回收时, 通过实现Drop让编译器自动处理
pub fn frame_alloc() -> Option<FrameTracker> {
    FRAME_ALLOCATOR
        .exclusive_access()
        .alloc()
        .map(FrameTracker::new)
    //.map(|ppn| FrameTracker::new(ppn))
}

fn frame_dealloc(ppn: PhysPageNum) {
    FRAME_ALLOCATOR.exclusive_access().dealloc(ppn);
}
