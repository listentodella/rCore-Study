use super::address::PhysPageNum;
use bitflags::*;

/*
Page Table Entry, 每个页表项为64b, 即8字节

Reserved: [63:54]
PPN[2]:   [53:28]
PPN[1]:   [27:19]
PPN[0]:   [18:10]
RSW:      [9:8]
DAGUXWRV: [7:0]

三级页表:
假设有虚拟地址 VA(由VPN_0, VPN_1, VPN_2, offset等组成)
- 首先会记录装载「当前所用的一级页表的物理页」的页号到 satp 寄存器中
- 把 VPN_0 作为偏移在一级页表的物理页中找到二级页表的物理页号
- 把 VPN_1 作为偏移在二级页表的物理页中找到三级页表的物理页号
- 把 VPN_2 作为偏移在三级页表的物理页中找到要访问位置的物理页号
物理页号对应的物理页基址（即物理页号左移12位）加上offset 就是VA对应的PA

*/

#[derive(Copy, Clone, Debug)]
#[repr(C)]
pub struct PageTableEntry {
    pub bits: usize,
}

bitflags! {
    #[derive(Debug, PartialEq)]
    pub struct PTEFlags:u8 {
        const V = 1 << 0;// 仅当V为1时, 页表项才是合法的
        const R = 1 << 1;//对应的虚拟页面是否可读
        const W = 1 << 2;//对应的虚拟页面是否可写
        const X = 1 << 3;//对应的虚拟页面是否可执行
        const U = 1 << 4;//对应的虚拟页面,在CPU处于U时是否可访问
        const G = 1 << 5;//reserve
        const A = 1 << 6;//Accessed,处理器动态地 记录自从页表项上的这一位被清零之后,页表项对应的虚拟页面是否被访问过
        const D = 1 << 7;//Dirty,处理器动态地 记录自从页表项上的这一位被清零之后,页表项对应的虚拟页面是否被修改过
    }
}

impl PageTableEntry {
    pub fn new(ppn: PhysPageNum, flags: PTEFlags) -> Self {
        PageTableEntry {
            bits: ppn.0 << 10 | flags.bits() as usize,
        }
    }

    pub fn empty() -> Self {
        Self { bits: 0 }
    }

    pub fn ppn(&self) -> PhysPageNum {
        (self.bits >> 10 & ((1usize << 44) - 1)).into()
    }
    pub fn flags(&self) -> PTEFlags {
        PTEFlags::from_bits(self.bits as u8).unwrap()
    }

    pub fn is_valid(&self) -> bool {
        (self.flags() & PTEFlags::V) != PTEFlags::empty()
    }
}
