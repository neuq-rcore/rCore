pub mod address;
pub mod frame;
pub mod heap;
pub mod page;

use alloc::{collections::BTreeMap, vec::Vec};
use core::{ops::Range, str};
use page::PageTable;

use bitflags::bitflags;

use crate::config::PAGE_SIZE;

use self::{
    address::{PhysPageNum, VirtAddr, VirtPageNum},
    frame::{frame_alloc, TrackedFrame},
    page::PageTableEntryFlags,
};

pub fn init() {
    heap::init();
    frame::init();
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum MapType {
    Identical,
    Framed,
}

// 表示一段连续的虚拟内存映射区域
pub struct MapArea {
    range: Range<VirtPageNum>,
    data_frames: BTreeMap<VirtPageNum, TrackedFrame>,
    map_type: MapType,
    permission: MapPermission,
}

impl MapArea {
    #[inline]
    pub fn vpn_range(&self) -> Range<usize> {
        // `step` trait was recently redisgned,
        // so we convert to usize first.
        let start = self.range.start.0;
        let end = self.range.end.0;
        start..end
    }
}

bitflags! {
    pub struct MapPermission: u8 {
        const R = 1 << 1;
        const W = 1 << 2;
        const X = 1 << 3;
        const U = 1 << 4;
    }
}

// 表示整个内存空间
pub struct MemorySpace {
    page_table: PageTable,
    areas: Vec<MapArea>,
}

impl MemorySpace {
    pub fn new_empty() -> Self {
        Self {
            page_table: PageTable::new(),
            areas: Vec::new(),
        }
    }

    fn push(&mut self, mut map_area: MapArea, data: Option<&[u8]>) {
        map_area.map_many(&mut self.page_table);
        if let Some(data) = data {
            map_area.copy_data(&mut self.page_table, data);
        }
        self.areas.push(map_area);
    }

    pub fn insert_framed_area(
        &mut self,
        start_va: VirtAddr,
        end_va: VirtAddr,
        permission: MapPermission,
    ) {
        let map_area = MapArea::new(start_va, end_va, MapType::Framed, permission);
        self.push(map_area, None);
    }
}

impl MapArea {
    pub fn new(
        start_va: VirtAddr,
        end_va: VirtAddr,
        map_type: MapType,
        permission: MapPermission,
    ) -> Self {
        let start_vpn = start_va.floor();
        let end_vpn = end_va.ceil();
        Self {
            range: start_vpn..end_vpn,
            data_frames: BTreeMap::new(),
            map_type,
            permission,
        }
    }

    pub fn map_page(&mut self, page_table: &mut PageTable, vpn: VirtPageNum) {
        let ppn = PhysPageNum(match self.map_type {
            MapType::Identical => vpn.0,
            MapType::Framed => {
                let frame = frame_alloc().unwrap();
                let ppn = frame.ppn.0;
                self.data_frames.insert(vpn, frame);
                ppn
            }
        });

        let flags = PageTableEntryFlags::from_bits(self.permission.bits()).unwrap();

        page_table.map(vpn, ppn, flags);
    }

    pub fn unmap_page(&mut self, page_table: &mut PageTable, vpn: VirtPageNum) {
        if self.map_type == MapType::Framed {
            let frame = self.data_frames.remove(&vpn).unwrap();
            // auto dealloc with deconstructor
        }

        page_table.unmap(vpn);
    }
}

impl MapArea {
    pub fn map_many(&mut self, page_table: &mut PageTable) {
        for vpn in self.vpn_range() {
            self.map_page(page_table, VirtPageNum(vpn));
        }
    }

    pub fn unmap_many(&mut self, page_table: &mut PageTable) {
        for vpn in self.vpn_range() {
            self.unmap_page(page_table, VirtPageNum(vpn));
        }
    }

    pub fn copy_data(&mut self, page_table: &mut PageTable, data: &[u8]) {
        let len = data.len();
        let mut copied = 0;

        let mut vpn: usize = self.range.start.0;

        while copied < len {
            let len = Ord::min(len, copied + PAGE_SIZE);
            let src = &data[copied..len];
            let dst = &mut page_table
                .translate(VirtPageNum(vpn))
                .unwrap()
                .ppn()
                .as_page_bytes_slice()[..len];

            dst.copy_from_slice(src);

            copied += PAGE_SIZE;
            vpn += 1;
        }
    }
}
