use super::{
    address::{VPNRange, VirtAddr, VirtPageNum},
    frame_allocator::FrameTracker,
    page_table::PageTable,
};
use crate::mm::address::{PhysAddr, PhysPageNum, StepByOne};
use crate::mm::frame_allocator::frame_alloc;
use crate::mm::page_table::PTEFlags;
use crate::{
    config::{MEMORY_END, PAGE_SIZE, TRAMPOLINE, TRAP_CONTEXT, USER_STACK_SIZE},
    sync::UPSafeCell,
};
use alloc::vec::Vec;
use alloc::{collections::btree_map::BTreeMap, sync::Arc};
use bitflags::bitflags;
use core::arch::asm;
use lazy_static::lazy_static;
use log::trace;
use riscv::register::satp;

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
    pub fn from_elf(elf_data: &[u8]) -> (Self, usize, usize) {
        let mut memory_set = Self::new_bare();
        // map trampoline, 将跳板插入到应用地址空间
        memory_set.map_trampoline();
        // map program headers of elf, with U flag
        //NOTE: xmas_elf crate的使用
        let elf = xmas_elf::ElfFile::new(elf_data).unwrap();
        let elf_header = elf.header;
        let magic = elf_header.pt1.magic;
        assert_eq!(magic, [0x7f, 0x45, 0x4c, 0x46], "invalid elf!");
        let ph_count = elf_header.pt2.ph_count();
        let mut max_end_vpn = VirtPageNum(0);
        for i in 0..ph_count {
            let ph = elf.program_header(i).unwrap();
            // 当 program header 的类型是LOAD时,才有被加载的必要
            if ph.get_type().unwrap() == xmas_elf::program::Type::Load {
                let start_va = (ph.virtual_addr() as usize).into();
                let end_va = ((ph.virtual_addr() + ph.mem_size()) as usize).into();
                let mut map_perm = MapPermission::U;
                let ph_flags = ph.flags();
                if ph_flags.is_read() {
                    map_perm |= MapPermission::R;
                }
                if ph_flags.is_write() {
                    map_perm |= MapPermission::W;
                }
                if ph_flags.is_execute() {
                    map_perm |= MapPermission::X;
                }
                let map_area = MapArea::new(start_va, end_va, MapType::Framed, map_perm);
                max_end_vpn = map_area.vpn_range.get_end();
                memory_set.push(
                    map_area,
                    Some(&elf.input[ph.offset() as usize..(ph.offset() + ph.file_size()) as usize]),
                );
            }
        }

        //map user stack with U flags
        let max_end_va: VirtAddr = max_end_vpn.into();
        let mut user_stack_bottom: usize = max_end_va.into();
        // guard page
        user_stack_bottom += PAGE_SIZE;
        let user_stack_top = user_stack_bottom + USER_STACK_SIZE;
        memory_set.push(
            MapArea::new(
                user_stack_bottom.into(),
                user_stack_top.into(),
                MapType::Framed,
                MapPermission::R | MapPermission::W | MapPermission::U,
            ),
            None,
        );
        // map TrapContext
        memory_set.push(
            MapArea::new(
                TRAP_CONTEXT.into(),
                TRAMPOLINE.into(),
                MapType::Framed,
                MapPermission::R | MapPermission::W,
            ),
            None,
        );

        (
            memory_set,
            user_stack_top,
            elf.header.pt2.entry_point() as usize,
        )
    }

    pub fn activate(&self) {
        let satp = self.page_table.token();
        unsafe {
            // 写satp的指令及其下一条指令,都在内核内存布局的代码段中
            // 在切换之后是一个恒等映射
            // 切换之前则是物理地址直接取指,也可以看作是恒等映射
            // 即使切换了地址空间,指令依然能够被连续执行
            satp::write(satp);
            // 一旦切换地址空间后,TLB里的旧映射关系就失效了
            // 通过该指令将TLB清空
            // 本质上该指令是一个barrier,
            //所有发生在它后面的地址转换都能够看到所有排在它前面的写入操作
            asm!("sfence.vma");
        }
    }
}

// 创建内核地址空间的全局实例
lazy_static! {
    pub static ref KERNEL_SPACE: Arc<UPSafeCell<MemorySet>> =
        Arc::new(unsafe { UPSafeCell::new(MemorySet::new_kernel()) });
}
