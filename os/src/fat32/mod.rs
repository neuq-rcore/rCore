mod virt;

use alloc::boxed::Box;

use virt::VirtioDisk;

use crate::driver::virt::VIRTIO0;
use fatfs::{
    FileSystem, FsOptions, IoBase, LossyOemCpConverter, NullTimeProvider, Read, Seek, SeekFrom,
    Write,
};
use virtio_drivers::{VirtIOBlk, VirtIOHeader};

pub struct Fat32FileSystem;

impl Fat32FileSystem {
    pub fn new(device_id: usize) -> FileSystem<Fat32IO, NullTimeProvider, LossyOemCpConverter> {
        let pa = VIRTIO0 + device_id * 0x1000;

        // Kernel space is identity mapped
        let va = pa;

        let header = unsafe { &mut *(va as *mut VirtIOHeader) };
        
        println!("[Disk] Valid: {:}", header.verify());
        assert!(header.verify(), "Header is not valid");

        let blk = VirtIOBlk::new(header).expect("Failed to create VirtIOBlk");

        let device = Box::new(VirtioDisk::new(blk));

        let io = Fat32IO::new(device);

        FileSystem::new(io, FsOptions::new()).unwrap()
    }
}

pub struct Fat32IO {
    device: Box<dyn IDiskDevice>,
}

impl Fat32IO {
    pub fn new(device: Box<dyn IDiskDevice>) -> Self {
        Fat32IO { device }
    }
}

impl IoBase for Fat32IO {
    type Error = ();
}

impl Read for Fat32IO {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, ()> {
        let device = &mut self.device;
        let device_offset = device.get_position() % 512;

        let size_read = if device_offset != 0 || buf.len() < 512 {
            let mut tmp = [0u8; 512];
            device.read_blocks(&mut tmp);

            let start = device_offset;
            let end = (device_offset + buf.len()).min(512);

            buf[..end - start].copy_from_slice(&tmp[start..end]);
            end - start
        } else {
            device.read_blocks(buf);
            512
        };

        device.move_cursor(size_read);
        Ok(size_read)
    }
}

impl Write for Fat32IO {
    fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        let device = &mut self.device;
        let device_offset = device.get_position() % 512;

        let size_written = if device_offset != 0 || buf.len() < 512 {
            let mut tmp = [0u8; 512];
            device.read_blocks(&mut tmp);

            let start = device_offset;
            let end = (device_offset + buf.len()).min(512);

            tmp[start..end].copy_from_slice(&buf[..end - start]);
            device.write_blocks(&tmp);
            end - start
        } else {
            device.write_blocks(buf);
            512
        };

        device.move_cursor(size_written);
        Ok(size_written)
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl Seek for Fat32IO {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64, Self::Error> {
        let device = &mut self.device;
        match pos {
            fatfs::SeekFrom::Start(i) => {
                device.set_position(i as usize);
                Ok(i)
            }
            fatfs::SeekFrom::Current(i) => {
                let new_pos = (device.get_position() as i64) + i;
                device.set_position(new_pos as usize);
                Ok(new_pos as u64)
            }
            fatfs::SeekFrom::End(_) => unreachable!(),
        }
    }
}

trait IDiskDevice {
    fn read_blocks(&mut self, buf: &mut [u8]);

    fn write_blocks(&mut self, buf: &[u8]);

    fn get_position(&self) -> usize;

    fn set_position(&mut self, position: usize);

    fn move_cursor(&mut self, amount: usize);
}
