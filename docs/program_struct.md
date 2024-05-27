# 项目架构

省略部分无关目录。

```shell
neuqOS
├── bootloader
│  ├── opensbi-qemu.bin
│  ├── rustsbi-k210.bin
│  └── rustsbi-qemu.bin
├── docs
│  ├── assets
│  │  ├── final_score_report.jpg
│  │  ├── neuq.jpg
│  │  └── visual_report.png
│  ├── content.md
│  ├── implements
│  │  ├── device_manager.md
│  │  ├── driver.md
│  │  ├── file_system.md
│  │  ├── file_system_api.md
│  │  ├── memory_manager.md
│  │  └── process_manager.md
│  ├── language.md
│  └── program_struct.md
├── LICENSE
├── log
├── Makefile
├── os
│  ├── Cargo.lock
│  ├── Cargo.toml
│  ├── Makefile
│  └── src
│     ├── allocation
│     │  └── mod.rs
│     ├── boards
│     │  ├── mod.rs
│     │  └── qemu.rs
│     ├── config.rs
│     ├── driver
│     │  ├── mod.rs
│     │  └── virt
│     │     └── mod.rs
│     ├── fat32
│     │  ├── mod.rs
│     │  └── virt.rs
│     ├── fs
│     │  ├── inode.rs
│     │  └── mod.rs
│     ├── lang_items.rs
│     ├── linker-qemu.ld
│     ├── logging.rs
│     ├── main.rs
│     ├── mm
│     │  ├── address.rs
│     │  ├── frame.rs
│     │  ├── heap.rs
│     │  ├── mod.rs
│     │  └── page.rs
│     ├── sbi
│     │  ├── console.rs
│     │  ├── mod.rs
│     │  ├── qemu.rs
│     │  └── system.rs
│     ├── stack_trace.rs
│     ├── stdio.rs
│     ├── sync
│     │  ├── mod.rs
│     │  └── up.rs
│     ├── syscall
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
│     ├── timer.rs
│     └── trap
│        ├── context.rs
│        ├── mod.rs
│        └── trap.S
├── prompt-generator.py
├── README.md
├── rust-toolchain.toml
├── test
├── thirdparty
└── user
   ├── build.py
   ├── Cargo.lock
   ├── Cargo.toml
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
      └── syscall.rs
```
