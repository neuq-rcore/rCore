pub trait IExt4Driver {
    fn read_blocks(&self, sector: usize, buf: &mut [u8]);

    fn write_blocks(&self, sector: usize, buf: &[u8]);
}