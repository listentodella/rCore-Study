use alloc::collections::btree_map::BTreeMap;
use bitflags::bitflags;

use super::{
    address::{VPNRange, VirtPageNum},
    frame_allocator::FrameTracker,
};

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
