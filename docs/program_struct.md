# 项目架构

省略部分无关目录。

```shell
neuqOS
├── bootloader
│  ├── opensbi-qemu.bin
│  ├── rustsbi-k210.bin
│  └── rustsbi-qemu.bin
├── docs # 文档
│  ├── assets
│  │  ├── neuq.jpg
│  │  └── visual_report.png
│  ├── content.md # 文档总目录
│  ├── implements
│  │  ├── device_manager.md
│  │  ├── file_system.md
│  │  ├── memory_manager.md
│  │  └── process_manager.md
│  ├── language.md
│  └── program_struct.md
├── LICENSE # MIT许可
├── Makefile
├── os # Super态
│  ├── Cargo.lock # cargo 依赖管理（自动生成）
│  ├── Cargo.toml # cargo 配置文件
│  ├── Makefile
│  └── src
│     ├── allocation
│     │  └── mod.rs
│     ├── boards # 板卡相关信息
│     │  ├── mod.rs
│     │  └── qemu.rs
│     ├── config.rs
│     ├── driver
│     │  ├── mod.rs
│     │  └── virt
│     │     └── mod.rs
│     ├── fat32 # FAT32 文件系统
│     │  ├── mod.rs
│     │  └── virt.rs
│     ├── fs # 文件系统
│     │  ├── inode.rs
│     │  └── mod.rs
│     ├── lang_items.rs
│     ├── linker-qemu.ld
│     ├── logging.rs
│     ├── main.rs
│     ├── mm # MMU
│     │  ├── address.rs
│     │  ├── frame.rs
│     │  ├── heap.rs
│     │  ├── mod.rs
│     │  └── page.rs
│     ├── sbi # sbi文件
│     │  ├── console.rs
│     │  ├── mod.rs
│     │  ├── qemu.rs
│     │  └── system.rs
│     ├── stack_trace.rs
│     ├── stdio.rs
│     ├── sync
│     │  ├── mod.rs
│     │  └── up.rs
│     ├── syscall # 系统调用
│     │  ├── fs.rs
│     │  ├── mod.rs
│     │  ├── process.rs
│     │  └── system.rs
│     ├── task
│     │  ├── context.rs
│     │  ├── mod.rs
│     │  ├── pid.rs
│     │  ├── processor.rs
│     │  ├── switch.rs
│     │  ├── switch.S
│     │  ├── task.rs
│     │  └── TaskManager.rs
│     ├── timer.rs # 计时器
│     └── trap # 异常处理
│        ├── context.rs
│        ├── mod.rs
│        └── trap.S
├── README.md
├── rust-toolchain.toml
├── test
├── thirdparty # 第三方依赖
└── user # User态
   ├── build.py
   ├── Cargo.lock # cargo 依赖管理（自动生成）
   ├── Cargo.toml # cargo 配置文件
   ├── Makefile
   └── src
      ├── bin
      │  ├── 00power_3.rs
      │  ├── 01power_5.rs
      │  ├── 02power_7.rs
      │  └── 03sleep.rs
      ├── console.rs
      ├── lang_items.rs
      ├── lib.rs
      ├── linker-qemu.ld
      └── syscall.rs # 系统调用
```
