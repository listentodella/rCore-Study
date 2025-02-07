use super::{
    address::{VPNRange, VirtAddr, VirtPageNum},
    frame_allocator::FrameTracker,
    page_table::{self, PageTable},
};
use crate::config::{MEMORY_END, PAGE_SIZE, TRAMPOLINE};
use crate::mm::address::{PhysAddr, PhysPageNum, StepByOne};
use crate::mm::frame_allocator::frame_alloc;
use crate::mm::page_table::PTEFlags;
use alloc::collections::btree_map::BTreeMap;
use alloc::vec::Vec;
use bitflags::bitflags;
use log::trace;

extern "C" {
    fn stext();
    fn etext();
    fn srodata();
    fn erodata();
    fn sdata();
    fn edata();
    fn sbss();
    fn sbss_with_stack();
    fn ebss();
    fn ekernel();
    fn strampoline();
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum MapType {
    // 恒等映射
    Identical,
    // 每个虚拟页面都有一个新分配的物理页帧对应
    // va 与 pa 的映射关系是随机的
    Framed,
}

bitflags! {
    pub struct MapPermission:u8 {
        const R = 1 << 1;
        const W = 1 << 2;
        const X = 1 << 3;
        const U = 1 << 4;
    }
}

pub struct MapArea {
    // 描述一段VPN的连续区间，表示该逻辑段在地址区间中的位置和长度。
    // 实现了iter
    vpn_range: VPNRange,
    data_frames: BTreeMap<VirtPageNum, FrameTracker>,
    map_type: MapType,
    map_perm: MapPermission,
}

impl MapArea {
    // 新建一个逻辑段结构体
    pub fn new(
        start_va: VirtAddr,
        end_va: VirtAddr,
        map_type: MapType,
        map_perm: MapPermission,
    ) -> Self {
        // 向下/上取整为虚拟页号
        let start_vpn = start_va.floor();
        let end_vpn = end_va.ceil();
        Self {
            vpn_range: VPNRange::new(start_vpn, end_vpn),
            data_frames: BTreeMap::new(),
            map_type,
            map_perm,
        }
    }

    // 将当前逻辑段到物理内存的映射,从传入的该逻辑段所属的地址空间的多级页表中加入或删除
    // 其实就是遍历逻辑段中的所有虚拟页面,进行map_one或unmap_one
    pub fn map(&mut self, page_table: &mut PageTable) {
        for vpn in self.vpn_range {
            self.map_one(page_table, vpn);
        }
    }
    pub fn unmap(&mut self, page_table: &mut PageTable) {
        for vpn in self.vpn_range {
            self.unmap_one(page_table, vpn);
        }
    }

    /// data: start-aligned but maybe with shorter length
    /// assume that all frames were cleared before
    // 将切片中的数据拷贝到当前逻辑段实际被内核放置在的各物理页帧上
    pub fn copy_data(&mut self, page_table: &PageTable, data: &[u8]) {
        assert_eq!(self.map_type, MapType::Framed);
        let mut start: usize = 0;
        let mut current_vpn = self.vpn_range.get_start();
        let len = data.len();
        loop {
            //Ausize.min(Busize)=> 返回AB中的较小值
            //即每次迭代,src的长度为 len 与 PAGE_SIZE 的较小值
            let src = &data[start..len.min(start + PAGE_SIZE)];
            let dst = &mut page_table
                .translate(current_vpn)
                // 获得PageTableEntry
                .unwrap()
                // 提取出ppn
                .ppn()
                // 获得ppn里对应的切片(固定4K大小, 但是要根据需求使用)
                .get_bytes_array()[..src.len()];

            // 复制到dst
            dst.copy_from_slice(src);
            start += PAGE_SIZE;
            if start >= len {
                break;
            }
            // 下一个虚拟页面
            current_vpn.step();
        }
    }
    //对逻辑段中的单个虚拟页面进行map
    pub fn map_one(&mut self, page_table: &mut PageTable, vpn: VirtPageNum) {
        let ppn: PhysPageNum;
        match self.map_type {
            //恒等映射,物理页号就等于虚拟页号
            MapType::Identical => {
                ppn = PhysPageNum(vpn.0);
            }
            //Framed 方式,需要分配一个物理页帧让当前的虚拟页面可以映射过去
            //此时物理页号就是 这个被分配的物理页帧的物理页号
            MapType::Framed => {
                let frame = frame_alloc().unwrap();
                ppn = frame.ppn;
                self.data_frames.insert(vpn, frame);
            }
        }
        // permission 转换到 PTEFlags
        let pte_flags = PTEFlags::from_bits(self.map_perm.bits()).unwrap();
        //调用多级页表 PageTable 的 map 接口来插入键值对
        page_table.map(vpn, ppn, pte_flags);
    }

    //对逻辑段中的单个虚拟页面进行unmap
    pub fn unmap_one(&mut self, page_table: &mut PageTable, vpn: VirtPageNum) {
        match self.map_type {
            MapType::Framed => {
                self.data_frames.remove(&vpn);
            }
            _ => {}
        }
        page_table.unmap(vpn);
    }
}

pub struct MemorySet {
    page_table: PageTable,
    areas: Vec<MapArea>,
}

impl MemorySet {
    // 创建一个空的地址空间
    pub fn new_bare() -> Self {
        Self {
            page_table: PageTable::new(),
            areas: Vec::new(),
        }
    }

    // 可以在当前地址空间插入一个新的逻辑段map_area
    fn push(&mut self, mut map_area: MapArea, data: Option<&[u8]>) {
        map_area.map(&mut self.page_table);
        // 目前只支持 Framed map_type 写入一些初始数据
        if let Some(data) = data {
            map_area.copy_data(&self.page_table, data);
        }
        self.areas.push(map_area);
    }

    //可以在当前地址空间插入一个 Framed 方式映射到物理内存的逻辑段
    //caller要保证同一地址空间内的任意两个逻辑段不能存在交集
    pub fn insert_framed_area(
        &mut self,
        start_va: VirtAddr,
        end_va: VirtAddr,
        permission: MapPermission,
    ) {
        self.push(
            MapArea::new(start_va, end_va, MapType::Framed, permission),
            None,
        );
    }

    /// Mention that trampoline is not collected by areas.
    fn map_trampoline(&mut self) {
        self.page_table.map(
            VirtAddr::from(TRAMPOLINE).into(),
            PhysAddr::from(strampoline as usize).into(),
            PTEFlags::R | PTEFlags::X,
        );
    }
    // without kernel stacks
    pub fn new_kernel() -> Self {
        let mut memory_set = Self::new_bare();
        // map trampoline
        memory_set.map_trampoline();
        // map kenrel stacks
        trace!(
            "[kernel] .text [{:#x}, {:#x})",
            stext as usize,
            etext as usize
        );
        trace!(
            "[kernel] .rodata [{:#x}, {:#x})",
            srodata as usize,
            erodata as usize
        );
        trace!(
            "[kernel] .data [{:#x}, {:#x})",
            sdata as usize,
            edata as usize
        );
        trace!(
            "[kernel] .bss [{:#x}, {:#x})",
            //sbss as usize,
            sbss_with_stack as usize,
            ebss as usize
        );
        trace!("[kernel] mapping .text section");
        memory_set.push(
            MapArea::new(
                (stext as usize).into(),
                (etext as usize).into(),
                MapType::Identical,
                MapPermission::R | MapPermission::X,
            ),
            None,
        );
        trace!("[kernel] mapping .rodata section");
        memory_set.push(
            MapArea::new(
                (srodata as usize).into(),
                (erodata as usize).into(),
                MapType::Identical,
                MapPermission::R,
            ),
            None,
        );
        trace!("[kernel] mapping .data section");
        memory_set.push(
            MapArea::new(
                (sdata as usize).into(),
                (edata as usize).into(),
                MapType::Identical,
                MapPermission::R | MapPermission::W,
            ),
            None,
        );
        trace!("[kernel] mapping .bss section");
        memory_set.push(
            MapArea::new(
                //(sbss as usize).into(),
                (sbss_with_stack as usize).into(),
                (ebss as usize).into(),
                MapType::Identical,
                MapPermission::R | MapPermission::W,
            ),
            None,
        );
        trace!("[kernel] mapping physical memory");
        memory_set.push(
            MapArea::new(
                (ekernel as usize).into(),
                MEMORY_END.into(),
                MapType::Identical,
                MapPermission::R | MapPermission::W,
            ),
            None,
        );
        memory_set
    }
    // pub fn from_elf(elf_data: &[u8]) -> (Self, usize, usize) {}
}
