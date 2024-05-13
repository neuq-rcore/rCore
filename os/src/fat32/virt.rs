use virtio_drivers::VirtIOBlk;

use crate::driver::virt::VirtioHal;

use super::IDiskDevice;

pub const SECTOR_SIZE: usize = 512;

pub struct VirtioDisk {
    sector: usize,
    offset: usize,
    virtio_blk: VirtIOBlk<'static, VirtioHal>,
}

impl VirtioDisk {
    pub fn new(virtio_blk: VirtIOBlk<'static, VirtioHal>) -> Self {
        VirtioDisk {
            sector: 0,
            offset: 0,
            virtio_blk,
        }
    }
}

impl IDiskDevice for VirtioDisk {
    fn read_blocks(&mut self, buf: &mut [u8]) {
        self.virtio_blk
            .read_block(self.sector, buf)
            .expect("Error occurred when reading VirtIOBlk");
    }

    fn write_blocks(&mut self, buf: &[u8]) {
        self.virtio_blk
            .write_block(self.sector, buf)
            .expect("Error occurred when writing VirtIOBlk");
    }

    fn get_position(&self) -> usize {
        self.sector * SECTOR_SIZE + self.offset
    }

    fn set_position(&mut self, position: usize) {
        self.sector = position / SECTOR_SIZE;
        self.offset = position % SECTOR_SIZE;
    }

    fn move_cursor(&mut self, amount: usize) {
        self.set_position(self.get_position() + amount)
    }
}
