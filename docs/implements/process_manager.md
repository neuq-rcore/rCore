# 进程管理

## 1. 概述

本文档旨在详细描述操作系统内核中的进程管理部分。进程管理是操作系统的核心功能之一，负责创建、调度、同步和终止进程。

## 2. 进程控制块 (TaskControlBlock)

进程控制块（TaskControlBlock，简称 TCB）是操作系统内核用来描述进程状态和控制进程运行的数据结构。

### 2.1 结构定义

```rust
pub struct TaskControlBlock {
    // 进程ID
    pub pid: PidHandle,
    // 内核栈
    pub kernel_stack: KernelStack,
    // 进程状态，包含就绪、运行、退出等
    pub task_status: TaskStatus,
    // 任务上下文，保存寄存器等状态
    pub task_ctx: TaskContext,
    // 内存空间管理
    pub memory_space: MemorySpace,
    // 陷阱上下文物理页号
    pub trap_ctx_ppn: PhysPageNum,
    // 基础大小
    pub base_size: usize,
    // 父进程
    pub parent: Option<Arc<TaskControlBlock>>,
    // 子进程列表
    pub children: Vec<Arc<TaskControlBlock>>,
    // 退出代码
    pub exit_code: i32,
    // 当前工作目录
    pub cwd: String,
    // 堆位置
    pub heap_pos: usize,
    // 文件描述符表
    pub fd_table: Vec<FileDescriptor>,
    // 内存映射工作区
    pub mmap_workaround: Vec<(usize, usize)>, // fd, ptr
}
```

### 2.2 状态管理

进程状态通过 `TaskStatus` 枚举管理：

```rust
pub enum TaskStatus {
    UnInit,    // 未初始化
    Ready,     // 就绪
    Running,   // 运行中
    Exited,    // 已退出
    Zombie,    // 僵尸状态
}
```

## 3. 任务上下文 (TaskContext)

任务上下程序 (TaskContext) 用于保存和恢复进程的执行上下文，如寄存器状态。

```rust
pub struct TaskContext {
    ra: usize,    // 返回地址
    sp: usize,    // 栈指针
    s: [usize; 12], // 保存的其他寄存器
}
```

## 4. 内存空间管理 (MemorySpace)

内存空间管理负责进程的内存管理，包括虚拟地址到物理地址的映射。

```rust
pub struct MemorySpace {
    page_table: PageTable, // 页表
    areas: Vec<MapArea>,   // 内存映射区域列表
}
```

## 5. 进程创建与调度

### 5.1 进程创建

进程创建通常从 `kernel_create_process` 函数开始，该函数通过解析 ELF 程序文件来创建新的进程。

```rust
pub fn kernel_create_process(elf_data: &[u8]) {
    let pcb = Arc::new(TaskControlBlock::new(elf_data, pid_alloc()));
    add_task(pcb);
}
```

### 5.2 调度器

调度器负责选择就绪状态的进程运行。进程调度通过 `schedule` 函数实现，该函数会触发任务上下文切换。

```rust
pub fn schedule(switched_task_cx_ptr: *mut TaskContext) {
    let mut processor = PROCESSOR.exclusive_access();
    let idle_task_cx_ptr = processor.get_idle_task_cx_ptr();
    drop(processor);
    unsafe {
        __switch(switched_task_cx_ptr, idle_task_cx_ptr);
    }
}
```

## 6. 进程同步

进程同步通过 `sync` 模块提供，支持多种同步原语，如互斥锁等。

## 7. 进程终止

进程终止通过 `exit_current_and_run_next` 函数实现，该函数会终止当前进程并切换到下一个就绪状态的进程。

```rust
pub fn exit_current_and_run_next(exit_code: i32) {
    // ...
}
```

## 8. 系统调用

系统调用是用户态程序请求内核态服务的一种方式，进程管理相关的系统调用包括但不限于：

- `sys_exit`：退出进程。
- `sys_yield`：主动放弃CPU。
- `sys_fork`：创建子进程。
- `sys_clone`：创建进程副本。
- `sys_exec`：执行新的程序。

## 9. 文件描述符管理

文件描述符管理通过 `FileDescriptor` 结构实现，每个进程有自己的文件描述符表。

```rust
pub struct FileDescriptor {
    pub flags: OpenFlags,
    pub path: String,
    pub file_type: FileType,
}
```

## 10. 进程调度 (Process Scheduling)

### 10.1. 概述

进程调度是操作系统内核中的一个关键功能，它负责决定哪个进程获得处理器资源。调度器根据进程的状态和优先级来动态地进行选择。

### 10.2. 调度器的设计

调度器的设计基于优先级队列和时间片轮转等策略，以确保公平性和效率。

### 10.3. 调度过程

调度过程通常涉及以下步骤：

- **选择进程**：从就绪队列中选择一个进程。
- **上下文切换**：保存当前进程的状态，并加载选定进程的状态。
- **启动进程**：将控制权转交给选定的进程。

### 10.4. 调度器的实现

调度器的实现涉及到多个组件：

- **任务控制块 (TaskControlBlock)**：保存进程的状态和上下文。
- **任务上下文 (TaskContext)**：保存和恢复进程的执行上下文。
- **处理器 (Processor)**：管理当前任务和就绪队列。

### 10.5. 调度器的接口

调度器提供的接口包括：

- **schedule**：执行进程调度，触发上下文切换。

### 10.6. 示例代码

```rust
pub fn schedule(switched_task_cx_ptr: *mut TaskContext) {
    let mut processor = PROCESSOR.exclusive_access();
    let idle_task_cx_ptr = processor.get_idle_task_cx_ptr();
    drop(processor);
    unsafe {
        __switch(switched_task_cx_ptr, idle_task_cx_ptr);
    }
}
```
