use crate::config::{PAGE_SIZE, PAGE_SIZE_BITS_WIDTH};

use super::page::PageTableEntry;

const SV39_PA_WIDTH: usize = 56;
const SV39_VA_WIDTH: usize = 39;
const PPN_WIDTH_SV39: usize = SV39_PA_WIDTH - PAGE_SIZE_BITS_WIDTH;
const VPN_WIDTH_SV39: usize = SV39_VA_WIDTH - PAGE_SIZE_BITS_WIDTH;

const VIRT_PAGE_NUM_WIDTH: usize = 9;
const VIRT_PAGE_NUM_MASK: usize = (1 << VIRT_PAGE_NUM_WIDTH) - 1;

// Region - PhysAddr
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct PhysAddr(pub usize);

impl From<usize> for PhysAddr {
    fn from(v: usize) -> Self {
        Self(v & ((1 << SV39_PA_WIDTH) - 1))
    }
}

impl From<PhysAddr> for usize {
    fn from(v: PhysAddr) -> Self {
        v.0
    }
}

impl PhysAddr {
    pub fn floor(&self) -> PhysPageNum {
        PhysPageNum(self.0 / PAGE_SIZE)
    }
    pub fn ceil(&self) -> PhysPageNum {
        if self.0 == 0 {
            PhysPageNum(0)
        } else {
            PhysPageNum((self.0 - 1 + PAGE_SIZE) / PAGE_SIZE)
        }
    }
    pub fn page_offset(&self) -> usize {
        self.0 & (PAGE_SIZE - 1)
    }
    pub fn aligned(&self) -> bool {
        self.page_offset() == 0
    }
}

impl From<PhysPageNum> for PhysAddr {
    fn from(v: PhysPageNum) -> Self {
        Self(v.0 << PAGE_SIZE_BITS_WIDTH)
    }
}

// End Region - PhysAddr

// Region - VirtAddr
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct VirtAddr(pub usize);

impl From<usize> for VirtAddr {
    fn from(v: usize) -> Self {
        Self(v & ((1 << SV39_VA_WIDTH) - 1))
    }
}

impl From<VirtAddr> for usize {
    fn from(v: VirtAddr) -> Self {
        if v.0 >= (1 << (SV39_VA_WIDTH - 1)) {
            v.0 | (!((1 << SV39_VA_WIDTH) - 1))
        } else {
            v.0
        }
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

impl From<VirtPageNum> for VirtAddr {
    fn from(v: VirtPageNum) -> Self {
        Self(v.0 << PAGE_SIZE_BITS_WIDTH)
    }
}

// End Region - VirtAddr

// Region - PhysPageNum
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct PhysPageNum(pub usize);

impl From<usize> for PhysPageNum {
    fn from(v: usize) -> Self {
        Self(v & ((1 << PPN_WIDTH_SV39) - 1))
    }
}

impl From<PhysPageNum> for usize {
    fn from(v: PhysPageNum) -> Self {
        v.0
    }
}

impl From<PhysAddr> for PhysPageNum {
    fn from(v: PhysAddr) -> Self {
        assert_eq!(v.page_offset(), 0);
        v.floor()
    }
}

impl PhysPageNum {
    pub fn as_page_bytes_slice(&self) -> &'static mut [u8] {
        let physaddr = PhysAddr::from(*self);
        unsafe { core::slice::from_raw_parts_mut(physaddr.0 as *mut u8, PAGE_SIZE) }
    }

    pub fn as_page_bytes_ptr(&self) -> *mut u8 {
        let physaddr = PhysAddr::from(*self);
        physaddr.0 as *mut u8
    }

    #[allow(invalid_reference_casting)]
    pub fn as_page_bytes_mut<T>(&self) -> &'static mut T {
        let physaddr = PhysAddr::from(*self);
        unsafe { &mut *(physaddr.0 as *mut T) }
    }

    pub fn as_entry_slice(&self) -> &'static mut [PageTableEntry] {
        let physaddr = PhysAddr::from(*self);
        unsafe {
            core::slice::from_raw_parts_mut(
                physaddr.0 as *mut PageTableEntry,
                PAGE_SIZE / core::mem::size_of::<PageTableEntry>(),
            )
        }
    }
}

// End Region - PhysPageNum

// Region - VirtPageNum
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct VirtPageNum(pub usize);

impl From<usize> for VirtPageNum {
    fn from(v: usize) -> Self {
        Self(v & ((1 << VPN_WIDTH_SV39) - 1))
    }
}

impl From<VirtPageNum> for usize {
    fn from(v: VirtPageNum) -> Self {
        v.0
    }
}

impl From<VirtAddr> for VirtPageNum {
    fn from(v: VirtAddr) -> Self {
        assert_eq!(v.page_offset(), 0);
        v.floor()
    }
}

impl VirtPageNum {
    // 取出虚拟页号的三级页索引
    // Returns: [PPN0, PPN1, PPN2]
    pub fn into_indices(&self) -> [usize; 3] {
        let mut vpn = self.0;
        let mut idx = [0usize; 3];
        for i in (0..3).rev() {
            idx[i] = vpn & VIRT_PAGE_NUM_MASK;
            vpn >>= VIRT_PAGE_NUM_WIDTH;
        }
        idx
    }
}

// End Region - VirtPageNum
