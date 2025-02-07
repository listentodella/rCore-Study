pub use crate::board::APP_BASE_ADDRESS;
pub use crate::board::APP_SIZE_LIMIT;
pub use crate::board::CLOCK_FREQ;

pub const MAX_APP_NUM: usize = 9;
pub const USER_STACK_SIZE: usize = 4096 * 2;
pub const KERNEL_STACK_SIZE: usize = 4096 * 2;
pub const KERNEL_HEAP_SIZE: usize = 0x30_0000;

// 页内偏移的位宽
pub const PAGE_SIZE_BITS: usize = 12;
// 每个页面的大小
pub const PAGE_SIZE: usize = 0x1000; //4096, 4K

// 之前是通过linker.ld的ekernel指明内核数据的终止物理地址
// 起始物理地址则设定为 0x8000_0000
// 在这里我们硬编码**整块**物理内存的终止物理地址, 即8MB
pub const MEMORY_END: usize = 0x8080_0000;

pub const TRAMPOLINE: usize = usize::MAX - PAGE_SIZE + 1;
