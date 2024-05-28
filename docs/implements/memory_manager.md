# 内存管理器

内存管理包括 **分页（ paging ）** 和 **虚拟内存（ virtual memory ）** 用于提高内存利用率和内存安全

实现了 SV39 多级页表。

## 相关依赖

- `buddy_system_allocator`

## 相关模块

- `mm`: 虚拟内存的实现。

- `heap`: 管理堆内存的分配。

- `address`: 物理内存、虚拟内存、物理内存页号、虚拟内存页号的抽象。

- `frame`: 物理页贞管理。

- `page`: 页表和页表项。

## 模块 `heap`

静态全局变量 `HEAP_ALLOCATOR` 是 `LockedHeap` 的实例，编译时会被链接至 `.bss.heap` 块，使用了 Attribute `#[global_allocator]` ，表示它作为 **全局内存分配器** 。

- **alloc_error_handler(layout: core::alloc::Layout)**: 使用了 Attribute `#[alloc_error_handler]` ，是处理内存分配异常的函数。

模块对外提供了堆内存初始化函数：

- **init()**: 初始化堆内存，占用空间大小为 `KERNEL_HEAP_SIZE`。

## 模块 `address`

`PhysAddr` 和 `VirtAddr` 分别是物理地址和虚拟地址，它们都可以类型转换为 `usize` 或者自身对应的 **页码类型（ PageNum ）** ，也可以从 `usize` 或者自身对应的 **页码类型** 转化而来，并且有着相同的函数：

- **floor(&self)**: 返回向上取整的页码。

- **ceil(&self)**: 返回向下取整的页码。

- **page_offset(&self)**：返回当前地址在页内的偏移量。

- **aligned(&self)**: 判断当前地址是否对齐页面的起始位置。

`PhysPageNum` 和 `VirtPageNum` 分别是物理页码和虚拟页码，它们都可以类型转换为 `usize` 或者自身对应的 **地址类型（ Addr ）** ，也可以从 `usize` 或者自身对应的 **地址类型** 转化而来，两者的函数不相同。

`PhysPageNum` 的函数如下：

- **as_page_bytes_slice(&self)**: 返回当前页码的全部内存空间。

- **as_page_bytes_ptr(&self)**: 返回当前页码的全部内存空间的引用。

- **as_page_bytes_mut<T>(&self)**: 获取当前页码的全部内存空间，作为某一种具体的类型后返回。

- **as_entry_slice(&self)**: 获取当前页码的全部内存空间，作为 `PageTableEntry` 后返回（当前页存储页表项）。

`VirtPageNum` 的函数如下：

- **into_indices(&self)**: 取出虚拟页号的三级页索引。

## 模块 `frame`

## 模块 `page`

## 模块 `mm`
