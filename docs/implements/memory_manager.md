# 内存管理器

内存管理包括 **分页（ paging ）** 和 **虚拟内存（ virtual memory ）** 用于提高内存利用率和内存安全

实现了 SV39 多级页表。

## 相关依赖

- `buddy_system_allocator`

## 相关模块

- `mm`: 虚拟内存的实现。

- `heap`: 管理堆内存的分配。

- `address`: 物理内存、虚拟内存、物理内存页号、虚拟内存页号的抽象。

- `frame`: 栈式物理页帧管理。

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

`TrackedFrame` 是对 `PhysPageNum` 的封装，实现了创建和回收 `PhysPageNum` 的自动化。

`StackedFrameAllocator` 结构体实现了 trait `IFrameAllocator` ：

```rust
pub struct StackedFrameAllocator {
    curr_page_num: usize,      // 空闲内存的起始物理页号
    end_page_num: usize,       // 空闲内存的结束物理页号
    recycled: VecDeque<usize>, // 回收的物理页号
}
```

```rust
trait IFrameAllocator {
    fn alloc(&mut self) -> Option<PhysPageNum>;
    fn dealloc(&mut self, ppn: PhysPageNum);
    fn alloc_contiguous(&mut self, count: usize) -> Option<Vec<PhysPageNum>>;
}
```

模块中三个顶层函数对 `StackedFrameAllocator` 的函数进行了封装，并对外公开：

- **frame_alloc()**: 分配一块内存空间创建新的物理页码并返回，如果有空闲页号则马上分配，否则以 `curr_page_num` 为起点开辟一块新的区域，如果页号耗尽则返回空值，表示失败。

- **frame_dealloc(ppn: PhysPageNum)**: 根据指定的物理页码释放对应的内存空间，底层原理是将给定的物理页码回收。

- **frame_alloc_contiguous(count: usize)**: 分配一块连续的内存空间创建 `count` 数量的物理页码并返回 Vec 对象，如果未分配的页号不足以创建则返回空值。

- **frame_dealloc_contiguous(start_ppn: PhysPageNum, count: usize)**: 根据起始页码和长度释放对应的内存空间。

其原生函数 **init(&mut self, lhs: PhysPageNum, rhs: PhysPageNum)** 和 **new()** 用于初始化自身。

```rust
lazy_static! {
    pub static ref FRAME_ALLOCATOR: UPSafeCell<StackedFrameAllocator> =
        unsafe { UPSafeCell::new(StackedFrameAllocator::new()) };
}
```

模块中顶层函数 **init()** 就是对 `FRAME_ALLOCATOR::init()` 的包装， FRAME_ALLOCATOR 维护的物理地址从 `ekernel` 标记开始直到 `MEMORY_END` 。

## 模块 `page`

`PageTableEntry` 结构体是页表项的抽象，维护了一段 64 位地址表示实际的页表项。

更加详细的数据结构抽象与类型定义如下：

| 位坐标 | 63 ～ 54 | 53 ～ 28 | 27 ~ 19 | 18~ 10 | 9 ~ 8 | 7 | 6 | 5 | 4 | 3 | 2 | 1 | 0 |
|--------|----------|----------|---------|--------|-------|---|---|---|---|---|---|---|---|
| 定义   | 保留段   | PPN[2]   | PPN[1]  | PPN[0] | RSW   | D | A | G | U | X | W | R | V |
| 位数   | 10       | 26       | 9       | 9      | 2     | 1 | 1 | 1 | 1 | 1 | 1 | 1 | 1 |

PPN 表示物理页码的三级页表， `DAGUXWRV` 是标志位，由 `PageTableEntryFlags` 位标志集合维护：

```rust
bitflags! {
    pub struct PageTableEntryFlags: u8 {
        /// V(Valid)：仅当位 V 为 1 时，页表项合法
        const V = 1 << 0;
        /// R(Read)/W(Write)/X(eXecute)：表示页表项对应的虚拟页面是否允许读/写/执行
        const R = 1 << 1;
        const W = 1 << 2;
        const X = 1 << 3;
        /// U(User)：表示页表项对应的虚拟页面在 CPU 处于 U 特权级时是否允许被访问
        const U = 1 << 4;
        const G = 1 << 5;
        /// A(Accessed)：处理器记录自从页表项上的这一位被清零之后，页表项的对应虚拟页面是否被访问过
        const A = 1 << 6;
        /// D(Dirty)：处理器记录自从页表项上的这一位被清零之后，页表项的对应虚拟页面是否被修改过
        const D = 1 << 7;
    }
}
```

以上对标记为的解释摘自 **rCore-Tuturial** 。

`PageTableEntry` 的原生函数如下：

- **new(ppn: PhysPageNum, flags: PageTableEntryFlags)**: 返回根据指定的物理页码和位标志集合构造的页表项。

- **empty()**: 返回空的页表项。

- **ppn(&self)**: 获取物理页码并返回。

- **flags(&self)**: 获取位标志并返回。

- **is_valid(&self)**: 判断页表项对应的虚拟页面是否合法。

- **readable(&self)**: 判断页表项对应的虚拟页面是否可读。

- **writable(&self)**: 判断页表项对应的虚拟页面是否可写。

- **executable(&self)**: 判断页表项对应的虚拟页面是否可执行。

`PageTable` 是一个庞大的结构体，用于管理多级页表，其定义如下：

```rust
pub struct PageTable {
    root_ppn: PhysPageNum,
    frames: Vec<TrackedFrame>,
}
```

`PageTable` 的函数有很多，与结构体属性相关的函数如下：

- **new()**: 构造函数，使用页帧分配器初始化根级物理页表。

- **root_ppn(&self)**: 返回根级物理页表。

获取页表项的函数如下：

- **get_entry(&self, vpn: VirtPageNum)**: 根据指定的虚拟页码从 `root_ppn` 开始查找对应的页表项，如果找不到则返回空值。

- **get_create_entry(&mut self, vpn: VirtPageNum)**: 根据指定的虚拟页码从 `root_ppn` 开始查找对应的页表项，如果找不到则创建位标志为 V 的页表项。

管理物理页码与虚拟页码的函数如下，它们保证了物理页和虚拟页的一一对应关系：

- **map(&mut self, vpn: VirtPageNum, ppn: PhysPageNum, flags: PageTableEntryFlags)**: 将 `get_entry` 返回的 `PageTableEntry` 指向 ppn ，并且加以 flag 位标志，构建物理页和虚拟也的映射关系。

- **unmap(&mut self, vpn: VirtPageNum)**: 解除指定虚拟页码和它对应的物理页码的映射关系，如果尝试解除一个没有映射关系的虚拟页码则会 panic 。

提供了一种类似 MMU 的手动查页表的方法：

- **from_token(satp: usize)**: 临时创建一个用于查询的 `PageTable` ，根据参数 `satp` 得到根节点的物理页号，`frame` 字段为空，实际不控制资源。

- **token(&self)**: 将 `root_ppn` 转化为 `satp` 格式返回。

以下是对外提供的接口：

- **translate(&self, vpn: VirtPageNum)**: 根据指定的虚拟页码返回其对应的页表项。

- **translate_va(&self, va: VirtAddr)**: 根据指定的虚拟地址返回其对应的物理地址。

- **translate_bytes(token: usize, buf: &[u8])**: 根据 token 按照给定虚拟缓冲区获取物理页面对应的内容并返回 Vec 。

- **translate_string(token: usize, ptr: *const u8, limit: usize)**: 根据 token 按照给定的指针和长度返回物理地址对应的字符串。

- **copy_to_space(token: usize, src: *const u8, dst: *mut u8, len: usize)**: 根据 token 将 `src` 所在的虚拟原缓冲区写入 `dst` 对应的物理缓冲区。

- **copy_from_space(token: usize, src: *const u8, dst: *mut u8, len: usize)**: 根据 token 将 `src` 所在的物理原缓冲区写入 `dst` 对应的虚拟缓冲区。

## 模块 `mm`

`MapType` 是枚举类型，表示映射方式，其定义如下：

```rust
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum MapType {
    Identical,  // 恒等映射
    Framed,     // 物理页帧分配
}
```

`MapPermission` 是位标志结构体，表示控制访问方式，仅保留 4 个[位标志](#模块-page)，其定义如下：

```rust
bitflags! {
    #[derive(Clone, Copy)]
    pub struct MapPermission: u8 {
        const R = 1 << 1;
        const W = 1 << 2;
        const X = 1 << 3;
        const U = 1 << 4;
    }
}
```

`MapArea` 结构体是对一段连续的虚拟内存映射区域的抽象：

```rust
pub struct MapArea {
    range: Range<VirtPageNum>,                          // 维护的虚拟页码范围
    data_frames: BTreeMap<VirtPageNum, TrackedFrame>,   // 虚拟页码和物理页码的映射关系
    map_type: MapType,                                  // 映射方式
    permission: MapPermission,                          // 控制访问方式
}
```

其函数如下：

- **vpn_range(&self)**: 返回虚拟页码范围。

- **from_another(them: &MapArea)**: 构造函数，返回传入 `MapArea` 的拷贝。

- **new(start_va: VirtAddr, end_va: VirtAddr, map_type: MapType, permission: MapPermission)**: 构造函数，其中 `start_va` 和 `end_va` 构成 range 。

- **map_page(&mut self, page_table: &mut PageTable, vpn: VirtPageNum)**: 为给定的虚拟页码分配物理页码并构建它们的映射关系。

- **unmap_page(&mut self, page_table: &mut PageTable, vpn: VirtPageNum)**: 释放给定的虚拟页码对应的物理页帧，解除映射关系。

- **map_many(&mut self, page_table: &mut PageTable)**: 为 `range` 内的所有虚拟页码分配物理页码，构建映射关系。

- **unmap_many(&mut self, page_table: &mut PageTable)**: 释放 `range` 内的所有虚拟页码对应的物理页帧，解除映射关系。

- **copy_data(&mut self, page_table: &mut PageTable, data: &[u8])**: 将一段内存数据通过虚拟内存的方式储存在内存中，为其分配虚拟页码和物理页码，以及映射关系。

`MemorySpace` 结构体表示整个内存空间，其定义如下：

```rust
pub struct MemorySpace {
    page_table: PageTable,  // 页表
    areas: Vec<MapArea>,    // 所有映射区域
}
```

其对外公开的函数如下：

- **table(&self)**: 返回 `page_table` 。

- **new_empty()**: 构造一个空的 `MemorySpace` 并返回。

- **token(&self)**: 返回 `page_table` 的 token 。

- **insert_framed_area(&mut self, start_va: VirtAddr, end_va: VirtAddr, permission: MapPermission)**: 插入一个 `MapArea` 映射区域。

- **map_trampoline(&mut self)**：构建 `trampoline` 段的映射关系，它并没有存在于 `area` 中。

- **remove_area_with_start_vpn(&mut self, start_vpn: VirtPageNum)**: 移除以 `start_vpn` 为起点的 `MapArea` 映射区域。

- **clear(&mut self)**: 清空所有 `MapArea` 映射区域。

- **from_existed_space(them_space: &MemorySpace)**: 构造函数，返回传入 `MemorySpace` 的拷贝。


