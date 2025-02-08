use memory_set::KERNEL_SPACE;

mod address;
pub mod frame_allocator;
mod heap_allocator;
mod memory_set;
mod page_table;

/// initiate heap allocator, frame allocator and kernel space
pub fn init() {
    heap_allocator::init_heap();
    //heap_allocator::heap_test();
    frame_allocator::init_frame_allocator();

    // 通过new_kernel创建一个内核地址空间,并用Arc<UPSafeCell<T>>封装起来
    // 然后通过exclusive_access获取一个&mut MemorySet
    // 然后通过activate将satp CSR进行设置,激活SV39分页模式
    KERNEL_SPACE.exclusive_access().activate();

    memory_set::remap_test();
}
