use core::panic;

use alloc::vec::Vec;

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
        *entry = PageTableEntry::new(ppn, flags);
    }

    pub fn unmap(&mut self, vpn: VirtPageNum) {
        match self.get_entry(vpn) {
            Some(entry) => {
                assert!(entry.is_valid()); // The page should be mapped
                *entry = PageTableEntry::empty();
            }
            None => panic!("unmap a unmapped page"),
        }
    }
}

impl PageTable {
    pub fn get_entry(&self, vpn: VirtPageNum) -> Option<&mut PageTableEntry> {
        let indices = vpn.into_indices();
        let mut ppn = self.root_ppn;

        let entry = Option::None;

        for layer in 0..3 {
            let index = indices[layer];
            let entries = ppn.as_entry_slice();
            let entry = &mut entries[index];

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

        for layer in 0..3 {
            let idx = indices[layer];
            let entries = ppn.as_entry_slice();
            entry = &mut entries[idx];

            if layer == 2 {
                return entry;
            }

            if !entry.is_valid() {
                let frame = frame_alloc().unwrap();
                let flags =
                    PageTableEntryFlags::V | PageTableEntryFlags::R | PageTableEntryFlags::W;
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
