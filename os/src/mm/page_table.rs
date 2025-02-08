use super::{
    address::{PhysPageNum, VirtPageNum},
    frame_allocator::{self, frame_alloc, FrameTracker},
};
use alloc::vec;
use alloc::vec::Vec;
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

pub struct PageTable {
    root_ppn: PhysPageNum,
    frames: Vec<FrameTracker>,
}

//每个应用的地址空间都对应一个不同的多级页表，这也就意味这不同页表的起始地址（即页表根节点的地址）是不一样的。
//因此 PageTable 要保存它根节点的物理页号 root_ppn 作为页表唯一的区分标志。
impl PageTable {
    pub fn new() -> Self {
        let frame = frame_allocator::frame_alloc().unwrap();
        Self {
            root_ppn: frame.ppn,
            frames: vec![frame],
        }
    }

    // 在多级页表找到一个虚拟页号对应的页表项的可变引用
    // 如果在遍历的过程中发现有节点尚未创建则会新建一个节点
    fn find_pte_create(&mut self, vpn: VirtPageNum) -> Option<&mut PageTableEntry> {
        let idxs = vpn.indexes();
        let mut ppn = self.root_ppn;
        let mut ret: Option<&mut PageTableEntry> = None;
        //for (i, item) in idxs.iter_mut().enumerate() {
        //let pte = &mut ppn.get_pte_array()[*item];
        for i in 0..3 {
            let pte = &mut ppn.get_pte_array()[idxs[i]];
            if i == 2 {
                ret = Some(pte);
                break;
            }

            if !pte.is_valid() {
                let frame = frame_alloc().unwrap();
                *pte = PageTableEntry::new(frame.ppn, PTEFlags::V);
                self.frames.push(frame);
            }
            ppn = pte.ppn();
        }
        ret
    }

    //当找不到合法叶子节点的时候不会新建叶子节点而是直接返回 None 即查找失败
    fn find_pte(&self, vpn: VirtPageNum) -> Option<&mut PageTableEntry> {
        let idxs = vpn.indexes();
        let mut ppn = self.root_ppn;
        let mut ret: Option<&mut PageTableEntry> = None;
        for i in 0..3 {
            let pte = &mut ppn.get_pte_array()[idxs[i]];
            if i == 2 {
                ret = Some(pte);
                break;
            }
            if !pte.is_valid() {
                return None;
            }
            ppn = pte.ppn();
        }
        ret
    }

    // 在多级页表中插入一个键值对:建立va pa的映射关系
    pub fn map(&mut self, vpn: VirtPageNum, ppn: PhysPageNum, flags: PTEFlags) {
        let pte = self.find_pte_create(vpn).unwrap();
        assert!(!pte.is_valid(), "vpn {:?} is mapped before mapping", vpn);
        *pte = PageTableEntry::new(ppn, flags | PTEFlags::V);
    }
    // 来删除一个键值对:拆除va pa的映射关系
    pub fn unmap(&mut self, vpn: VirtPageNum) {
        let pte = self.find_pte(vpn).unwrap();
        assert!(!pte.is_valid(), "vpn {:?} is invalid before unmapping", vpn);
        *pte = PageTableEntry::empty();
    }

    // 可以临时创建一个专用来手动查找页表的PageTable
    // 它仅有一个从传入的satp token 中得到的多级页表根节点的物理页号
    // frames 字段为空, 即实际不控制任何资源
    pub fn from_token(satp: usize) -> Self {
        Self {
            root_ppn: PhysPageNum::from(satp & ((1usize << 44) - 1)),
            frames: Vec::new(),
        }
    }

    pub fn translate(&self, vpn: VirtPageNum) -> Option<PageTableEntry> {
        self.find_pte(vpn).map(|pte| pte.clone())
    }

    pub fn token(&self) -> usize {
        8usize << 60 | self.root_ppn.0
    }
}
