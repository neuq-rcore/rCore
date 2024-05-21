use alloc::{slice, vec::Vec};

use crate::config::PAGE_SIZE;

use super::{
    address::{PhysAddr, PhysPageNum, VirtAddr, VirtPageNum},
    frame::{frame_alloc, TrackedFrame},
};
use bitflags::*;

pub struct PageTable {
    root_ppn: PhysPageNum,
    frames: Vec<TrackedFrame>,
}

impl PageTable {
    pub fn new() -> Self {
        let root = frame_alloc().unwrap();

        Self {
            root_ppn: root.ppn,
            frames: vec![root],
        }
    }

    pub fn root_ppn(&self) -> PhysPageNum {
        self.root_ppn
    }
}

impl PageTable {
    pub fn map(&mut self, vpn: VirtPageNum, ppn: PhysPageNum, flags: PageTableEntryFlags) {
        let entry = self.get_create_entry(vpn);
        assert!(!entry.is_valid()); // 一个虚拟页只能映射到一个物理页
        *entry = PageTableEntry::new(ppn, flags | PageTableEntryFlags::V);
        assert!(entry.is_valid());
    }

    pub fn unmap(&mut self, vpn: VirtPageNum) {
        let entry = self
            .get_entry(vpn)
            .expect("Attempted to unmap an unmapped page");
        *entry = PageTableEntry::empty();
    }
}

impl PageTable {
    pub fn get_entry(&self, vpn: VirtPageNum) -> Option<&mut PageTableEntry> {
        let indices = vpn.into_indices();
        let mut ppn = self.root_ppn;

        let entry = Option::None;

        for (layer, &idx) in indices.iter().enumerate() {
            let entries = ppn.as_entry_slice();
            let entry = &mut entries[idx];

            if layer == 2 {
                return Option::Some(entry);
            }

            if !entry.is_valid() {
                return Option::None;
            }

            ppn = entry.ppn();
        }

        entry
    }

    pub fn get_create_entry(&mut self, vpn: VirtPageNum) -> &mut PageTableEntry {
        let indices = vpn.into_indices();
        let mut ppn = self.root_ppn;

        let mut entry: &mut PageTableEntry;

        for (layer, &idx) in indices.iter().enumerate() {
            let entries = ppn.as_entry_slice();
            entry = &mut entries[idx];

            if layer == 2 {
                return entry;
            }

            if !entry.is_valid() {
                let frame = frame_alloc().unwrap();
                let flags = PageTableEntryFlags::V;
                *entry = PageTableEntry::new(frame.ppn, flags);
                self.frames.push(frame);
            }

            ppn = entry.ppn();
        }

        unreachable!();
    }
}

impl PageTable {
    pub fn translate(&self, vpn: VirtPageNum) -> Option<PageTableEntry> {
        self.get_entry(vpn).map(|entry| *entry)
    }

    pub fn translate_va(&self, va: VirtAddr) -> Option<PhysAddr> {
        self.get_entry(va.clone().floor()).map(|pte| {
            let aligned_pa: PhysAddr = pte.ppn().into();
            let offset = va.page_offset();
            let aligned_pa_usize: usize = aligned_pa.into();
            (aligned_pa_usize + offset).into()
        })
    }

    pub fn translate_bytes(token: usize, buf: &[u8]) -> Vec<&'static [u8]> {
        let page_table = PageTable::from_token(token);
        let mut start = buf.as_ptr() as usize;
        let end = start + buf.len();

        let mut v = Vec::new();

        while start < end {
            let start_va = VirtAddr::from(start);
            let mut vpn = start_va.floor();

            let ppn = page_table.translate(vpn).unwrap().ppn();

            vpn = VirtPageNum(vpn.0 + 1);
            let mut end_va: VirtAddr = vpn.into();
            end_va = end_va.min(VirtAddr::from(end));
            if end_va.page_offset() == 0 {
                v.push(&ppn.as_page_bytes_slice()[start_va.page_offset()..]);
            } else {
                v.push(&ppn.as_page_bytes_slice()[start_va.page_offset()..end_va.page_offset()]);
            }
            start = end_va.into();
        }

        v
    }

    pub fn copy_to_space(token: usize, src: *const u8, dst: *mut u8, len: usize) -> usize {
        let page_table = PageTable::from_token(token);
        let start = dst as usize;
        let mut copied_bytes = 0;

        while copied_bytes < len {
            let start_va = VirtAddr::from(dst as usize + copied_bytes);
            let mut vpn = start_va.floor();

            let ppn = page_table.translate(vpn).unwrap().ppn();

            // step in
            vpn = VirtPageNum(vpn.0 + 1);

            let mut end_va: VirtAddr = vpn.into();
            end_va = end_va.min(VirtAddr::from(start + len));

            let end_va_offset = end_va.page_offset();
            let start_va_offset = start_va.page_offset();

            let bytes_this_page = if end_va_offset == 0 {
                PAGE_SIZE - start_va_offset
            } else {
                end_va_offset - start_va_offset
            };

            let src = unsafe {
                slice::from_raw_parts((src as usize + copied_bytes) as *const u8, bytes_this_page)
            };
            let dst =
                &mut ppn.as_page_bytes_slice()[start_va_offset..start_va_offset + bytes_this_page];

            dst.copy_from_slice(src);

            copied_bytes += bytes_this_page;
        }

        copied_bytes
    }

    pub fn copy_from_space(token: usize, src: *const u8, dst: *mut u8, len: usize) -> usize {
        let page_table = PageTable::from_token(token);
        let start = src as usize;
        let mut copied_bytes = 0;


        while copied_bytes < len {
            let start_va = VirtAddr::from(src as usize + copied_bytes);
            let mut vpn = start_va.floor();

            let ppn = page_table.translate(vpn).unwrap().ppn();

            // step in
            vpn = VirtPageNum(vpn.0 + 1);

            let mut end_va: VirtAddr = vpn.into();
            end_va = end_va.min(VirtAddr::from(start + len));

            let end_va_offset = end_va.page_offset();
            let start_va_offset = start_va.page_offset();

            let bytes_this_page = if end_va_offset == 0 {
                PAGE_SIZE - start_va_offset
            } else {
                end_va_offset - start_va_offset
            };

            let src = &ppn.as_page_bytes_slice()[start_va_offset..start_va_offset + bytes_this_page];
            let dst = unsafe { slice::from_raw_parts_mut((dst as usize + copied_bytes) as *mut u8, bytes_this_page) };

            dst.copy_from_slice(src);

            copied_bytes += bytes_this_page;
        }

        copied_bytes
    }

    pub fn from_token(stap: usize) -> Self {
        let root_ppn = PhysPageNum(stap & ((1 << 44) - 1));
        Self {
            root_ppn,
            frames: Vec::new(),
        }
    }

    pub fn token(&self) -> usize {
        8usize << 60 | self.root_ppn.0
    }
}

#[derive(Copy, Clone)]
#[repr(C)]
/// page table entry structure
pub struct PageTableEntry {
    pub bits: usize,
}

impl PageTableEntry {
    pub fn new(ppn: PhysPageNum, flags: PageTableEntryFlags) -> Self {
        PageTableEntry {
            bits: ppn.0 << 10 | flags.bits() as usize,
        }
    }

    pub fn empty() -> Self {
        PageTableEntry { bits: 0 }
    }

    pub fn ppn(&self) -> PhysPageNum {
        (self.bits >> 10 & ((1usize << 44) - 1)).into()
    }

    pub fn flags(&self) -> PageTableEntryFlags {
        PageTableEntryFlags::from_bits(self.bits as u8).unwrap()
    }

    pub fn is_valid(&self) -> bool {
        self.flags().contains(PageTableEntryFlags::V)
    }

    pub fn readable(&self) -> bool {
        self.flags().contains(PageTableEntryFlags::R)
    }

    pub fn writable(&self) -> bool {
        self.flags().contains(PageTableEntryFlags::W)
    }

    pub fn executable(&self) -> bool {
        self.flags().contains(PageTableEntryFlags::X)
    }
}

bitflags! {
    /// page table entry flags
    pub struct PageTableEntryFlags: u8 {
        const V = 1 << 0;
        const R = 1 << 1;
        const W = 1 << 2;
        const X = 1 << 3;
        const U = 1 << 4;
        const G = 1 << 5;
        const A = 1 << 6;
        const D = 1 << 7;
    }
}
