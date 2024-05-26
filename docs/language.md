# Program Language

## Rust

大部分常见的操作系统内核基于 `C 语言` ，neuqOS使用 `Rust` 进行开发，原因如下：

- **安全性**： Rust 是一种内存安全的编程语言，其借用检查器（Borrow Checker）可以在编译时捕获访问错误，如内存安全、线程安全等。避免了常见的内存安全问题，如空指针引用、数据竞争等。

- **性能**： Rust 语言的设计目标之一是提供与 C 和 C++ 相媲美的性能。它具有零开销抽象、内联汇编和对底层硬件的直接访问等特性，使得开发人员能够编写高效的系统级代码。
- **并发**： Rust支持多种并发模型，如线程、通道、锁等。这使得Rust在处理多线程程序时具有更好的性能。
- 易于维护：Rust的代码更加简洁，易于阅读和维护。Rust还提供了许多工具，如代码补全、错误检查等，有助于提高开发效率。

- **可靠性**： Rust 鼓励编写可靠的代码。其强制的所有权和借用规则以及丰富的静态类型系统可以在编译时捕获潜在的错误，减少运行时错误的可能性。

- **丰富的第三方库支持**： Rust生态系统中有大量的高质量第三方库，可以帮助简化内核开发过程。

尽管C语言是广泛用于操作系统开发的传统语言，但Rust也是一种功能强大、性能高、易于维护的编程语言，适用于开发操作系统内核。

## Toolchains

- `rustc 1.77.0-nightly` (bf3c6c5be 2024-02-01)

- `cargo 1.77.0-nightly` (7bb7b5395 2024-01-20)

## Build Dependencies

- `riscv64-elf-gcc`

- `riscv64-elf-binutils`

- `cargo-binutils`

- `llvm-tools-preview`

Refer to [`rust-toolchain.toml`](../rust-toolchain.toml)

For command line instructions, refer to [`.github/workflows/ci.yml`](../.github/workflows/ci.yml)