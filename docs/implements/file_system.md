# 文件系统

文件系统实现了从磁盘读取数据的功能。

## 相关依赖

- `virtio-drivers`

## 相关模块

- `fat32`: 提供创建文件系统的入口

  - `virt`: 与磁盘进行 IO 通信

## 子模块 `virt`

- `VirtioDisk` 结构体是对操作磁盘简单操作的工具，实现了 trait `IDiskDevice` ( 在 fat32 模块定义 ) ，对外提供了以下函数：

  - **new(virtio_blk: VirtIOBlk<VirtioHal, MmioTransport>)**: 创建一个 `VirtioDisk` 对象。

  - **read_blocks(&mut self, buf: &mut [u8])**: 读取块。

  - **write_blocks(&mut self, buf: &[u8])**：写入块。

  - **get_position(&self)**：获得当前游标位置。

  - **set_position(&mut self, position: usize)**: 移动游标到指定位置。

  - **move_cursor(&mut self, amount: usize)**: 根据给定偏移量向前移动游标。

## 模块 `fat32` 

- `Fat32IO` 结构体实现了 trait `Read` 、 `Write` 、 `Seek` ，增加了对磁盘的复杂操作，对外提供了以下函数：

  - **read_exact(&mut self, mut buf: &mut [u8])**: 读取一个 512KB 大小的块到 `buf` ，容量不够会抛出异常。

  - **read(&mut self, buf: &mut [u8])**: 对 `read_exact` 进行包装，处理潜在的异常。

  - **write(&mut self, buf: &[u8])**: 将 512KB 大小的 `buf` 写入至当前的块中。

  - **flush(&mut self)**: 强制将缓冲区的内容写入磁盘，目前暂时为空实现。

  - **seek(&mut self, pos: SeekFrom)**: 移动当前游标至 **起始/当前/末尾** 加上偏移量

## 使用方式

在文件系统 API 中，使用如下方式创建一个文件系统入口：

```rust
let fs = Fat32FileSystem::new(0);
```
