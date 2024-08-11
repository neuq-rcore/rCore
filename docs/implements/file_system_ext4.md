## 技术文档：Ext4 文件系统实现

### 模块及依赖项

```rust
mod virt;

use core::ptr::NonNull;
use alloc::boxed::Box;
use virt::VirtioDisk;
use crate::driver::virt::{VirtioHal, VIRTIO0};
use ext4_rs::{
    FileSystem, FsOptions, IoBase, LossyOemCpConverter, NullTimeProvider, Read, Seek, SeekFrom,
    Write,
};
use virtio_drivers::device::blk::VirtIOBlk;
use virtio_drivers::transport::mmio::{MmioTransport, VirtIOHeader};
```

### 结构体 `Ext4FileSystem`

#### 概述
`Ext4FileSystem` 是用于表示和管理 Ext4 文件系统的结构体。它通过设备 ID 初始化并绑定到具体的 VirtIO 块设备上。

#### 方法

- `new(device_id: usize) -> FileSystem<Ext4IO, NullTimeProvider, LossyOemCpConverter>`:
  - 创建并初始化一个新的 Ext4 文件系统实例。
  - 参数 `device_id` 用于指定 VirtIO 设备的标识符。
  - 通过 MMIO 传输层与 VirtIO 设备通信，最终创建并返回一个 `FileSystem` 对象。

### 结构体 `Ext4IO`

#### 概述
`Ext4IO` 是用于与底层设备进行 I/O 操作的结构体，它实现了 `IoBase`、`Read`、`Write` 和 `Seek` 等 trait。

#### 方法

- `new(device: Box<dyn IDiskDevice>) -> Self`:
  - 构造函数，接受一个实现了 `IDiskDevice` trait 的设备对象。
  - 返回 `Ext4IO` 实例。

- `read_inner(&mut self, buf: &mut [u8]) -> Result<usize, ()>`:
  - 内部读取函数，用于从设备读取数据到缓冲区 `buf`。
  - 设备每次只能读取 512 字节，因此会根据需要处理部分读取。
  - 返回读取的字节数。

- `read(&mut self, buf: &mut [u8]) -> Result<usize, ()>`:
  - 实现 `Read` trait 的方法，用于读取数据到缓冲区。
  - 返回读取的字节数。

- `write(&mut self, buf: &[u8]) -> Result<usize, Self::Error>`:
  - 实现 `Write` trait 的方法，用于将缓冲区数据写入设备。
  - 返回写入的字节数。

- `flush(&mut self) -> Result<(), Self::Error>`:
  - 实现 `Write` trait 的方法，用于刷新缓冲区。

- `seek(&mut self, pos: SeekFrom) -> Result<u64, Self::Error>`:
  - 实现 `Seek` trait 的方法，用于调整文件指针位置。
  - 支持从开始、当前位置和文件结尾进行定位。

### Trait `IDiskDevice`

#### 概述
`IDiskDevice` 是一个抽象接口，用于定义基本的磁盘操作。

#### 方法

- `read_blocks(&mut self, buf: &mut [u8])`:
  - 从设备读取数据块到缓冲区。

- `write_blocks(&mut self, buf: &[u8])`:
  - 将缓冲区数据块写入设备。

- `get_position(&self) -> usize`:
  - 获取当前设备指针的位置。

- `set_position(&mut self, position: usize)`:
  - 设置设备指针的位置。

- `move_cursor(&mut self, amount: usize)`:
  - 移动设备指针位置。