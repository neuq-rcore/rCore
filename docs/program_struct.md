# 项目架构

```
.
├── bootloader
│  ├── opensbi-qemu.bin  
│  ├── rustsbi-k210.bin
│  └── rustsbi-qemu.bin
├── docs
│  ├── README.md
│  └── ...
├── LICENSE
├── Makefile
├── os
│  ├── Cargo.lock
│  ├── Cargo.toml
│  ├── Makefile
│  └── src
│     ├── boards 板卡相关信息
│     │  ├── mod.rs
│     │  └── qemu.rs
│     ├── config.rs
│     ├── driver
│     │  ├── mod.rs
│     │  └── virt
│     │     └── mod.rs
│     ├── fat32  FAT32文件实现
│     │  ├── mod.rs
│     │  └── virt.rs
│     ├── lang_items.rs
│     ├── linker-qemu.ld
│     ├── loader.rs
│     ├── logging.rs
│     ├── main.rs
│     ├── mm mmu实现
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
│     ├── syscall 系统调用实现
│     │  ├── fs.rs
│     │  ├── mod.rs
│     │  ├── process.rs
│     │  └── system.rs
│     ├── task 
│     │  ├── context.rs
│     │  ├── mod.rs
│     │  ├── switch.rs
│     │  ├── switch.S
│     │  └── task.rs
│     ├── timer.rs 定时实现
│     └── trap  trap实现
│        ├── context.rs
│        ├── mod.rs
│        └── trap.S
├── README.md
├── rust-toolchain.toml
├── sdcard.img
├── test
├── test_close.txt
├── test_mmap.txt
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
