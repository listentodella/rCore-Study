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
