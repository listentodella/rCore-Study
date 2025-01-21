use core::{alloc::Layout, ops::RangeBounds};

use buddy_system_allocator::LockedHeap;

use crate::config::KERNEL_HEAP_SIZE;

// LockedHeap 是被Mutex保护的类型
// 因此操作它前必须先获取锁
// 这也意味着多线程情况下,可能会有竞争的问题
#[global_allocator]
static HEAP_ALLOCATOR: LockedHeap<32> = LockedHeap::empty();

#[alloc_error_handler]
pub fn handle_alloc_error(layout: Layout) -> ! {
    panic!("[kernel] Heap alloc failed, layout {:?}", layout);
}

// 由于是static mut, 当前会被链接到.bss
static mut HEAP_SPACE: [u8; KERNEL_HEAP_SIZE] = [0; KERNEL_HEAP_SIZE];

pub fn init_heap() {
    unsafe {
        HEAP_ALLOCATOR
            .lock()
            .init(HEAP_SPACE.as_ptr() as usize, KERNEL_HEAP_SIZE);
    }
}

#[allow(unused)]
pub fn heap_test() {
    use alloc::boxed::Box;
    use alloc::vec::Vec;
    extern "C" {
        fn sbss();
        fn ebss();
    }
    let bss_range = sbss as usize..ebss as usize;
    let a = Box::new(5);

    assert_eq!(*a, 5);
    // 能够通过assert, 说明这段堆空间已可以被动态分配
    // 因为上面的Box分配了一个5i32, 确实落在这片堆空间中
    assert!(bss_range.contains(&(a.as_ref() as *const _ as usize)));
    drop(a);

    let mut v: Vec<usize> = Vec::new();
    for i in 0..500 {
        v.push(i);
    }

    for (i, _) in v.iter().enumerate().take(500) {
        assert_eq!(v[i], i);
    }

    // 同理, 该Vec也是如此
    assert!(bss_range.contains(&(v.as_ptr() as usize)));
    drop(v);
    println!("heap_test passed!");
}
