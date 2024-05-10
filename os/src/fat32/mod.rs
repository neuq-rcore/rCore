// /*
//     Since we have not implemented memory allocator, we can not alloc on the heap.
//     The code is heavily coupled with virtio driver.
// */

// mod device;

// use fatfs::{
//     FileSystem, IoBase, IoError, LossyOemCpConverter, NullTimeProvider, Read, Seek, Write,
// };

// use self::device::VirtIODevice;

// pub struct Fat32FileSystem;

// impl Fat32FileSystem {
//     pub fn new(
//         device_id: usize,
//     ) -> FileSystem<Fat32Provider, NullTimeProvider, LossyOemCpConverter> {
//         let provider = Fat32Provider {
//             device: VirtIODevice::new(device_id),
//         };
//         fatfs::FileSystem::new(provider, fatfs::FsOptions::new()).expect("open fs wrong")
//     }
// }

// pub struct Fat32Provider {
//     device: VirtIODevice,
// }

// impl Fat32Provider {
//     pub fn new(device_id: usize) -> Fat32Provider {
//         // TODO
//         todo!()
//     }
// }

// impl IoBase for Fat32Provider {
//     type Error = ();
// }

// impl Read for Fat32Provider {
//     fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
//         todo!()
//     }

//     fn read_exact(&mut self, mut buf: &mut [u8]) -> Result<(), Self::Error> {
//         while !buf.is_empty() {
//             match self.read(buf) {
//                 Ok(0) => break,
//                 Ok(n) => {
//                     let tmp = buf;
//                     buf = &mut tmp[n..];
//                 }
//                 Err(ref e) if e.is_interrupted() => {}
//                 Err(e) => return Err(e),
//             }
//         }
//         if buf.is_empty() {
//             Ok(())
//         } else {
//             log::debug!("failed to fill whole buffer in read_exact");
//             Err(Self::Error::new_unexpected_eof_error())
//         }
//     }
// }

// impl Write for Fat32Provider {
//     fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
//         todo!()
//     }

//     fn flush(&mut self) -> Result<(), Self::Error> {
//         todo!()
//     }
// }

// impl Seek for Fat32Provider {
//     fn seek(&mut self, pos: fatfs::SeekFrom) -> Result<u64, Self::Error> {
//         todo!()
//     }
// }
