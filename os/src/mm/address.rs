use crate::config::{PAGE_SIZE, PAGE_SIZE_BITS};

#[derive(Debug, Copy, Clone, Ord, PartialEq, Eq, PartialOrd)]
pub struct PhysAddr(pub usize);

#[derive(Debug, Copy, Clone, Ord, PartialEq, Eq, PartialOrd)]
pub struct VirtAddr(pub usize);

#[derive(Debug, Copy, Clone, Ord, PartialEq, Eq, PartialOrd)]
pub struct PhysPageNum(pub usize);

#[derive(Debug, Copy, Clone, Ord, PartialEq, Eq, PartialOrd)]
pub struct VirtPageNum(pub usize);

/*
virtual address (39bits)
virtual page number: [38:12]
page offset: [11:0]
-----
physical address (56bits)
physical page number: [55:12]
page offset: [11:0]

地址转换是以页为单位进行的，在地址转换的前后地址的页内偏移部分不变。
可以认为 MMU 只是从虚拟地址中取出 27 位虚拟页号,
在页表中查到其对应的物理页号（如果存在的话），
最后将得到的44位的物理页号与虚拟地址的12位页内偏移依序拼接到一起就变成了56位的物理地址
*/

//SV39 支持的物理地址位宽为 56 位，因此在生成 PhysAddr 的时候我们仅使用 usize 较低的 56 位
const PA_WIDTH_SV39: usize = 56;

// 56 - 12 = 44;
// 即44位用于标识PhysPageNum,即物理页号
// 后12位用于标识PageSize, 目前即2^12=4K
const PPN_WIDTH_SV39: usize = PA_WIDTH_SV39 - PAGE_SIZE_BITS;

impl From<usize> for PhysAddr {
    fn from(value: usize) -> Self {
        //SV39 支持的物理地址位宽为 56 位，因此在生成 PhysAddr 的时候我们仅使用 usize 较低的 56 位
        Self(value & ((1 << PA_WIDTH_SV39) - 1))
    }
}

impl From<usize> for PhysPageNum {
    fn from(value: usize) -> Self {
        Self(value & ((1 << PPN_WIDTH_SV39) - 1))
    }
}

impl From<PhysAddr> for usize {
    fn from(value: PhysAddr) -> Self {
        value.0
    }
}

impl From<PhysPageNum> for usize {
    fn from(value: PhysPageNum) -> Self {
        value.0
    }
}

impl PhysAddr {
    pub fn page_offset(&self) -> usize {
        self.0 & (PAGE_SIZE - 1)
    }

    // 向下取整
    pub fn floor(&self) -> PhysPageNum {
        PhysPageNum(self.0 / PAGE_SIZE)
    }

    // 向上取整
    pub fn ceil(&self) -> PhysPageNum {
        //PhysPageNum((self.0 + PAGE_SIZE - 1) / PAGE_SIZE)
        PhysPageNum(self.0.div_ceil(PAGE_SIZE))
    }
}

// 通过PhysAddr获取PhysPageNum
impl From<PhysAddr> for PhysPageNum {
    fn from(value: PhysAddr) -> Self {
        // 物理地址需要保证它与页面大小对齐才能转换
        assert_eq!(value.page_offset(), 0);
        value.floor()
    }
}

// 通过PhysPageNum获取PhysAddr
impl From<PhysPageNum> for PhysAddr {
    fn from(value: PhysPageNum) -> Self {
        Self(value.0 << PAGE_SIZE_BITS)
    }
}

impl PhysPageNum {
    pub fn get_bytes_array(&self) -> &'static mut [u8] {
        let pa: PhysAddr = self.clone().into();
        unsafe { core::slice::from_raw_parts_mut(pa.0 as *mut u8, 4096) }
    }
}
