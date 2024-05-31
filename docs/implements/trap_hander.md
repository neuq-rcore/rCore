# 中断处理

# 相关模块

- `trap`: 中断处理

  - `context`：上下文切换

# 子模块 `context`

`TrapContext` 结构体用于保存中断上下文，其定义如下：

```rust
#[repr(C)]
pub struct TrapContext {
    /// RISC-V 32 个寄存器
    pub x: [usize; 32],
    /// 管理器状态寄存器（Supervisor Status Register）
    pub sstatus: Sstatus,
    /// PC 寄存器，表示当前执行的指令位置
    pub sepc: usize,
    /// SATP ，与虚拟内存有关
    pub kernel_token: usize,
    /// 堆栈指针
    pub kernel_sp: usize,
    /// 中断处理器
    pub trap_handler: usize,
}
```

其函数如下：

- **set_sp(&mut self, sp: usize)**: 设置堆栈指针（x[2] 寄存器）。

- **app_init_context(entry: usize, sp: usize, kernel_token: usize, kernel_sp: usize, trap_handler: usize)**: 构造函数， `sstatus` 的特权级设置为 U 。

# 模块 `trap`

核心的函数是两个汇编函数：

- **__snap_trap()**: 将栈上程序的当前状态保存至 `.text.trampoline` 。

- **__restore_snap**: 将 `.text.trampoline` 内容恢复到栈上。

因此在程序自身的角度，在开始执行后，就仿佛一直在执行，直到停止。

`KernelTrapContext` 结构体是内核中断的上下文，其函数如下：

- **enter()**: 设置内核 trap 模式。

模块中其他公开函数如下：

- **trap_return()**: 将控制权转回到 User 模式。

- **on_kernel_trap()**: 响应系统级中断处理，跳转到 `kernel_trap_intenral()` 函数，因此没有返回值。

- **kernel_trap_intenral()**: 处理内核中断的函数，抛出异常。

- **set_user_trap()**: 设置用户 trap 模式。

- **disable_timer_interrupt()**: 禁用时钟中断器。

- **enable_timer_interrupt()**: 启用时钟中断器。

- **trap_handler()**: 处理 trap ，不同的 Trap 类型有不同的处理逻辑，如果是异常就中断程序，如果是时钟中断则挂起。
