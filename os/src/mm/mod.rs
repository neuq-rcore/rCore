pub mod address;
pub mod frame;
pub mod heap;
pub mod page;

use crate::{
    boards::MMIO,
    config::{TRAMPOLINE, TRAP_CONTEXT, USER_STACK_SIZE},
    sync::UPSafeCell,
};
use alloc::{collections::BTreeMap, sync::Arc, vec::Vec};
use core::{arch::asm, ops::Range, str};
use log::debug;
use page::PageTable;
use riscv::register::satp;

use bitflags::bitflags;
use lazy_static::lazy_static;

use crate::config::{MEMORY_END, PAGE_SIZE};

pub use self::{
    address::{PhysAddr, PhysPageNum, VirtAddr, VirtPageNum},
    frame::{frame_alloc, TrackedFrame},
};

use page::PageTableEntryFlags;

pub fn init() {
    heap::init();
    frame::init();

    KernelSpace::activate();
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
    pub fn table(&self) -> &PageTable {
        &self.page_table
    }

    pub fn new_empty() -> Self {
        Self {
            page_table: PageTable::new(),
            areas: Vec::new(),
        }
    }

    pub fn token(&self) -> usize {
        self.page_table.token()
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

    pub fn map_trampoline(&mut self) {
        extern "C" {
            fn strampoline();
        }

        self.page_table.map(
            VirtAddr::from(TRAMPOLINE).into(),
            PhysAddr::from(strampoline as usize).into(),
            PageTableEntryFlags::R | PageTableEntryFlags::X ,
        );
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
            let _frame = self.data_frames.remove(&vpn).unwrap();
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
            use log::*;
            info!("len: {}, copied: {}", len, copied);

            let len = Ord::min(len, copied + PAGE_SIZE);
            let src = &data[copied..len];
            let dst = &mut page_table
                .translate(VirtPageNum(vpn))
                .unwrap()
                .ppn()
                .as_page_bytes_slice()[..len - copied];

            dst.copy_from_slice(src);

            copied += PAGE_SIZE;
            vpn += 1;
        }
    }
}

pub struct KernelSpace;

impl KernelSpace {
    pub fn new() -> MemorySpace {
        extern "C" {
            fn stext();
            fn etext();

            fn srodata();
            fn erodata();

            fn sdata();
            fn edata();

            // do not use this as this does not contains the stack
            // fn sbss();
            fn ebss();

            fn ekernel();
        }

        let mut kernel_space = MemorySpace::new_empty();

        kernel_space.map_trampoline();

        debug!(
            "Mapping .text, 0x{:08X}..0x{:08X}",
            stext as usize, etext as usize
        );
        kernel_space.push(
            MapArea::new(
                VirtAddr(stext as usize),
                VirtAddr(etext as usize),
                MapType::Identical,
                MapPermission::R | MapPermission::X,
            ),
            Option::None,
        );

        debug!(
            "Mapping .rodata, 0x{:08X}..0x{:08X}",
            srodata as usize, erodata as usize
        );
        kernel_space.push(
            MapArea::new(
                VirtAddr(srodata as usize),
                VirtAddr(erodata as usize),
                MapType::Identical,
                MapPermission::R,
            ),
            Option::None,
        );

        debug!(
            "Mapping .data, 0x{:08X}..0x{:08X}",
            sdata as usize, edata as usize
        );
        kernel_space.push(
            MapArea::new(
                VirtAddr(sdata as usize),
                VirtAddr(edata as usize),
                MapType::Identical,
                MapPermission::R | MapPermission::W,
            ),
            Option::None,
        );

        debug!(
            "Mapping .bss, 0x{:08X}..0x{:08X}",
            edata as usize, ebss as usize
        );
        kernel_space.push(
            MapArea::new(
                VirtAddr(edata as usize),
                VirtAddr(ebss as usize),
                MapType::Identical,
                MapPermission::R | MapPermission::W,
            ),
            Option::None,
        );

        debug!(
            "Mapping physical memory, 0x{:08X}..0x{:08X}",
            ekernel as usize, MEMORY_END
        );
        kernel_space.push(
            MapArea::new(
                VirtAddr(ekernel as usize),
                VirtAddr(MEMORY_END),
                MapType::Identical,
                MapPermission::R | MapPermission::W,
            ),
            None,
        );

        debug!("Mapping memory-mapped registers");
        for &(start, len) in MMIO {
            debug!("Mapping MMIO: start: 0x{:08X}, len: 0x{:08X}", start, len);
            kernel_space.push(
                MapArea::new(
                    VirtAddr(start),
                    VirtAddr(start + len),
                    MapType::Identical,
                    MapPermission::R | MapPermission::W,
                ),
                None,
            );
        }

        kernel_space
    }

    pub fn activate() {
        let satp = kernel_token();

        debug!("Activating kernel space, SATP: 0x{:X}", satp);
        unsafe {
            satp::write(satp);
            asm!("sfence.vma");
        }
        debug!("Kernel space activated");
    }
}

lazy_static! {
    /// a memory set instance through lazy_static! managing kernel space
    pub static ref KERNEL_SPACE: Arc<UPSafeCell<MemorySpace>> =
        Arc::new(unsafe { UPSafeCell::new(KernelSpace::new()) });
}

pub fn kernel_token() -> usize {
    KERNEL_SPACE.exclusive_access().token()
}

pub struct UserSpace;

impl UserSpace {
    pub fn from_elf(elf_data: &[u8]) -> (MemorySpace, usize, usize) {
        let mut user_space = MemorySpace::new_empty();

        user_space.map_trampoline();

        let elf = xmas_elf::ElfFile::new(elf_data).unwrap();
        let header = elf.header;

        let magic = header.pt1.magic;
        // 0x7f 'E' 'L' 'F'
        assert!(
            magic == [0x7F, 0x45, 0x4C, 0x46],
            "Invalid ELF magic number"
        );

        let mut max_end_vpn = VirtPageNum(0);

        for ph in elf.program_iter() {
            debug!("ph: {:?}", ph);
            if ph.get_type().unwrap() != xmas_elf::program::Type::Load {
                continue;
            }

            let start_va = VirtAddr(ph.virtual_addr() as usize);
            let end_va = VirtAddr(ph.virtual_addr() as usize + ph.mem_size() as usize);

            let mut permission = MapPermission::U;
            let ph_flags = ph.flags();
            if ph_flags.is_read() {
                permission |= MapPermission::R;
            }
            if ph_flags.is_write() {
                permission |= MapPermission::W;
            }
            if ph_flags.is_execute() {
                permission |= MapPermission::X;
            }

            let map_area = MapArea::new(start_va, end_va, MapType::Framed, permission);

            max_end_vpn = Ord::max(max_end_vpn, end_va.ceil());

            let end = (ph.offset() + ph.file_size()) as usize;

            user_space.push(map_area, Some(&elf_data[ph.offset() as usize..end]));
        }
        debug!("End of ELF segments");

        // map user stack with U flags
        let max_end_va: VirtAddr = max_end_vpn.into();
        let mut user_stack_bottom: usize = max_end_va.into();

        // guard page
        user_stack_bottom += PAGE_SIZE;

        let user_stack_top = user_stack_bottom + USER_STACK_SIZE;
        debug!("Mapping user stack 0x{:08X}..0x{:08X}", user_stack_bottom, user_stack_top);
        user_space.push(
            MapArea::new(
                user_stack_bottom.into(),
                user_stack_top.into(),
                MapType::Framed,
                MapPermission::R | MapPermission::W | MapPermission::U,
            ),
            None,
        );

        debug!("Mapping WTF");
        // used in sbrk
        user_space.push(
            MapArea::new(
                user_stack_top.into(),
                user_stack_top.into(),
                MapType::Framed,
                MapPermission::R | MapPermission::W | MapPermission::U,
            ),
            None,
        );

        debug!("Mapping user trap context at 0x{:08X}", TRAP_CONTEXT);
        // map trap context with U flags
        user_space.push(
            MapArea::new(
                TRAP_CONTEXT.into(),
                TRAMPOLINE.into(),
                MapType::Framed,
                MapPermission::R | MapPermission::W,
            ),
            None,
        );

        extern "C" {
            fn strampoline();
        }

        // To allow instructions in `__restore_snap` to be executed
        user_space.page_table.map(
            VirtAddr::from(strampoline as usize).into(),
            PhysAddr::from(strampoline as usize).into(),
            PageTableEntryFlags::R | PageTableEntryFlags::X,
        );

        (
            user_space,
            // reserve 8 bits for a register
            user_stack_top - 8,
            elf.header.pt2.entry_point() as usize,
        )
    }
}
