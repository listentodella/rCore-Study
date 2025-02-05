mod address;
pub mod frame_allocator;
mod heap_allocator;
mod memory_set;
mod page_table;

/// initiate heap allocator, frame allocator and kernel space
pub fn init() {
    heap_allocator::init_heap();
    heap_allocator::heap_test();
}
