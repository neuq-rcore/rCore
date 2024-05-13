// // static DEVICE =

// use virtio_drivers::{Hal, VirtIOBlk};

// use crate::sync::UPSafeCell;

// pub trait IDevice {
//     fn read_block(&self, buf: &mut [u8]);

//     fn write_block(&self, buf: &[u8]);
// }

// const SECTOR_SIZE: usize = 512;

// pub const MMIO: &[(usize, usize)] = &[(0x10001000, 0x1000)];

// pub struct VirtIODevice {
//     sector: usize,
//     offset: usize,
//     device_id: usize,
//     block: UPSafeCell<VirtIOBlk<'static, VirtioHal>>,
// }

// impl VirtIODevice {
//     pub fn new(device_id: usize) -> VirtIODevice {
//         VirtIODevice {
//             sector: 0,
//             offset: 0,
//             device_id,
//             block: todo!(),
//         }
//     }

//     #[inline]
//     fn get_position(&self) -> usize {
//         (self.sector * SECTOR_SIZE) as usize + self.offset
//     }

//     fn set_position(&mut self, position: usize) {
//         self.sector = position / SECTOR_SIZE;
//         self.offset = position % SECTOR_SIZE;
//     }

//     fn move_cursor(&mut self, amount: usize) {
//         self.set_position(self.get_position() + amount)
//     }
// }

// impl IDevice for VirtIODevice {
//     fn read_block(&self, buf: &mut [u8]) {
//         self.block
//             .exclusive_access()
//             .read_block(self.sector, buf)
//             .expect("Error occurred when reading VirtIOBlk");
//     }

//     fn write_block(&self, buf: &[u8]) {
//         self.block
//             .exclusive_access()
//             .write_block(self.sector, buf);
//     }
// }

// pub struct VirtioHal;

// impl Hal for VirtioHal {
//     fn dma_alloc(pages: usize) -> usize {
//         let mut ppn_base = PhysPageNum(0);
//         for i in 0..pages {
//             let frame = frame_alloc().unwrap();
//             if i == 0 {
//                 ppn_base = frame.ppn;
//             }
//             assert_eq!(frame.ppn.0, ppn_base.0 + i);
//             QUEUE_FRAMES.exclusive_access().push(frame);
//         }
//         let pa: PhysAddr = ppn_base.into();
//         pa.0
//     }

//     fn dma_dealloc(pa: usize, pages: usize) -> i32 {
//         let pa = PhysAddr::from(pa);
//         let mut ppn_base: PhysPageNum = pa.into();
//         for _ in 0..pages {
//             frame_dealloc(ppn_base);
//             ppn_base.step();
//         }
//         0
//     }

//     fn phys_to_virt(addr: usize) -> usize {
//         addr
//     }

//     fn virt_to_phys(vaddr: usize) -> usize {
//         PageTable::from_token(kernel_token())
//             .translate_va(VirtAddr::from(vaddr))
//             .unwrap()
//             .0
//     }
// }
