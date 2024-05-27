use alloc::{collections::VecDeque, vec::Vec};
use lazy_static::lazy_static;

use crate::config::MEMORY_END;
use crate::sync::UPSafeCell;

use super::address::{PhysAddr, PhysPageNum};

trait IFrameAllocator {
    fn alloc(&mut self) -> Option<PhysPageNum>;
    fn dealloc(&mut self, ppn: PhysPageNum);
    fn alloc_contiguous(&mut self, count: usize) -> Option<Vec<PhysPageNum>>;
}

lazy_static! {
    pub static ref FRAME_ALLOCATOR: UPSafeCell<StackedFrameAllocator> =
        unsafe { UPSafeCell::new(StackedFrameAllocator::new()) };
}

pub fn init() {
    extern "C" {
        fn ekernel();
    }
    FRAME_ALLOCATOR.exclusive_access().init(
        PhysAddr::from(ekernel as usize).ceil(),
        PhysAddr::from(MEMORY_END).floor(),
    );
}

pub struct TrackedFrame {
    pub ppn: PhysPageNum,
}

// Auto dealloc frame when drop
impl Drop for TrackedFrame {
    fn drop(&mut self) {
        frame_dealloc(self.ppn);
    }
}

impl TrackedFrame {
    pub fn new(ppn: PhysPageNum) -> Self {
        // Clean a page
        ppn.as_page_bytes_slice().fill(0u8);

        Self { ppn }
    }
}

pub fn frame_alloc() -> Option<TrackedFrame> {
    FRAME_ALLOCATOR
        .exclusive_access()
        .alloc()
        .map(TrackedFrame::new)
}

pub fn frame_dealloc(ppn: PhysPageNum) {
    FRAME_ALLOCATOR.exclusive_access().dealloc(ppn);
}

pub fn frame_alloc_contiguous(count: usize) -> Option<Vec<TrackedFrame>> {
    FRAME_ALLOCATOR
        .exclusive_access()
        .alloc_contiguous(count)
        .map(|x| x.iter().map(|&t| TrackedFrame::new(t)).collect())
}

pub fn frame_dealloc_contiguous(start_ppn: PhysPageNum, count: usize) {
    let mut allocator = FRAME_ALLOCATOR.exclusive_access();

    for i in 0..count {
        allocator.dealloc((start_ppn.0 + i).into());
    }
}

// 栈式物理页帧管理策略
pub struct StackedFrameAllocator {
    curr_page_num: usize,      // 空闲内存的起始物理页号
    end_page_num: usize,       // 空闲内存的结束物理页号
    recycled: VecDeque<usize>, // 回收的物理页号
}

impl IFrameAllocator for StackedFrameAllocator {
    fn alloc(&mut self) -> Option<PhysPageNum> {
        if let Some(ppn) = self.recycled.pop_front() {
            return Some(PhysPageNum(ppn));
        }

        if self.curr_page_num == self.end_page_num {
            None
        } else {
            let ppn = Some(PhysPageNum(self.curr_page_num));
            self.curr_page_num += 1;

            ppn
        }
    }

    fn dealloc(&mut self, ppn: PhysPageNum) {
        let ppn = ppn.0;

        if ppn >= self.curr_page_num
            || self
                .recycled
                .iter()
                .any(|&recycled_ppn| recycled_ppn == ppn)
        {
            panic!("Frame {:?} is not allocated", ppn);
        }

        self.recycled.push_back(ppn);
    }

    fn alloc_contiguous(&mut self, count: usize) -> Option<Vec<PhysPageNum>> {
        if self.curr_page_num + count >= self.end_page_num {
            None
        } else {
            self.curr_page_num += count;
            let arr: Vec<usize> = (1..count + 1).collect();
            let v = arr
                .iter()
                .map(|x| (self.curr_page_num - x).into())
                .collect();
            Some(v)
        }
    }
}

impl StackedFrameAllocator {
    fn new() -> Self {
        Self {
            curr_page_num: 0,
            end_page_num: 0,
            recycled: VecDeque::new(),
        }
    }

    pub fn init(&mut self, lhs: PhysPageNum, rhs: PhysPageNum) {
        self.curr_page_num = lhs.0;
        self.end_page_num = rhs.0;
    }
}
