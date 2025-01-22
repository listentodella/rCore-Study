mod address;
mod heap_allocator;

/// initiate heap allocator, frame allocator and kernel space
pub fn init() {
    heap_allocator::init_heap();
    heap_allocator::heap_test();
}
