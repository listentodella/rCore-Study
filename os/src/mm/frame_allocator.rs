use core::panic;

use alloc::vec::Vec;

use super::address::PhysPageNum;

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
