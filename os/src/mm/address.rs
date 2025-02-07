use super::page_table::PageTableEntry;
use crate::config::{PAGE_SIZE, PAGE_SIZE_BITS};
use core::fmt::{self, Debug, Formatter};

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
const VA_WIDTH_SV39: usize = 39;

// 56 - 12 = 44;
// 即44位用于标识PhysPageNum,即物理页号
// 后12位用于标识PageSize, 目前即2^12=4K
const PPN_WIDTH_SV39: usize = PA_WIDTH_SV39 - PAGE_SIZE_BITS;
const VPN_WIDTH_SV39: usize = VA_WIDTH_SV39 - PAGE_SIZE_BITS;

impl From<usize> for PhysAddr {
    fn from(value: usize) -> Self {
        //SV39 支持的物理地址位宽为 56 位，因此在生成 PhysAddr 的时候我们仅使用 usize 较低的 56 位
        Self(value & ((1 << PA_WIDTH_SV39) - 1))
    }
}

impl From<usize> for VirtAddr {
    fn from(v: usize) -> Self {
        Self(v & ((1 << VA_WIDTH_SV39) - 1))
    }
}

impl From<usize> for VirtPageNum {
    fn from(v: usize) -> Self {
        Self(v & ((1 << VPN_WIDTH_SV39) - 1))
    }
}

impl From<VirtAddr> for VirtPageNum {
    fn from(v: VirtAddr) -> Self {
        assert_eq!(v.page_offset(), 0);
        v.floor()
    }
}
impl From<VirtPageNum> for VirtAddr {
    fn from(v: VirtPageNum) -> Self {
        Self(v.0 << PAGE_SIZE_BITS)
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

impl VirtAddr {
    pub fn floor(&self) -> VirtPageNum {
        VirtPageNum(self.0 / PAGE_SIZE)
    }
    pub fn ceil(&self) -> VirtPageNum {
        if self.0 == 0 {
            VirtPageNum(0)
        } else {
            VirtPageNum((self.0 - 1 + PAGE_SIZE) / PAGE_SIZE)
        }
    }
    pub fn page_offset(&self) -> usize {
        self.0 & (PAGE_SIZE - 1)
    }
    pub fn aligned(&self) -> bool {
        self.page_offset() == 0
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
    //返回一个字节数组(4K)的可变引用，可以以字节为粒度对物理页帧上的数据进行访问
    pub fn get_bytes_array(&self) -> &'static mut [u8] {
        let pa: PhysAddr = self.clone().into();
        unsafe { core::slice::from_raw_parts_mut(pa.0 as *mut u8, 4096) }
    }

    //返回一个页表项定长数组的可变引用，代表多级页表中的一个节点
    pub fn get_pte_array(&self) -> &'static mut [PageTableEntry] {
        let pa: PhysAddr = self.clone().into();
        unsafe { core::slice::from_raw_parts_mut(pa.0 as *mut PageTableEntry, 512) }
    }

    //可以获取一个恰好放在一个物理页帧开头的类型为 T 的数据的可变引用
    pub fn get_mut<T>(&self) -> &'static mut T {
        let pa: PhysAddr = self.clone().into();
        unsafe { (pa.0 as *mut T).as_mut().unwrap() }
    }
}

impl VirtPageNum {
    // 取出虚拟页号的三级页索引, 并按照从高到低的顺序返回
    pub fn indexes(&self) -> [usize; 3] {
        let mut vpn = self.0;
        let mut idx = [0usize; 3];
        for i in (0..3).rev() {
            idx[i] = vpn & 511;
            vpn >>= 9;
        }
        idx
    }
}

pub trait StepByOne {
    fn step(&mut self);
}
impl StepByOne for VirtPageNum {
    fn step(&mut self) {
        self.0 += 1;
    }
}

#[derive(Copy, Clone)]
/// a simple range structure for type T
pub struct SimpleRange<T>
where
    T: StepByOne + Copy + PartialEq + PartialOrd + Debug,
{
    l: T,
    r: T,
}
impl<T> SimpleRange<T>
where
    T: StepByOne + Copy + PartialEq + PartialOrd + Debug,
{
    pub fn new(start: T, end: T) -> Self {
        assert!(start <= end, "start {:?} > end {:?}!", start, end);
        Self { l: start, r: end }
    }
    pub fn get_start(&self) -> T {
        self.l
    }
    pub fn get_end(&self) -> T {
        self.r
    }
}
impl<T> IntoIterator for SimpleRange<T>
where
    T: StepByOne + Copy + PartialEq + PartialOrd + Debug,
{
    type Item = T;
    type IntoIter = SimpleRangeIterator<T>;
    fn into_iter(self) -> Self::IntoIter {
        SimpleRangeIterator::new(self.l, self.r)
    }
}
/// iterator for the simple range structure
pub struct SimpleRangeIterator<T>
where
    T: StepByOne + Copy + PartialEq + PartialOrd + Debug,
{
    current: T,
    end: T,
}
impl<T> SimpleRangeIterator<T>
where
    T: StepByOne + Copy + PartialEq + PartialOrd + Debug,
{
    pub fn new(l: T, r: T) -> Self {
        Self { current: l, end: r }
    }
}
impl<T> Iterator for SimpleRangeIterator<T>
where
    T: StepByOne + Copy + PartialEq + PartialOrd + Debug,
{
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        if self.current == self.end {
            None
        } else {
            let t = self.current;
            self.current.step();
            Some(t)
        }
    }
}

/// a simple range structure for virtual page number
pub type VPNRange = SimpleRange<VirtPageNum>;
