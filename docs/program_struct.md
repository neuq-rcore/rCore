# 项目架构

省略部分无关目录。

```shell
neuqOS
├── bootloader
│  ├── opensbi-qemu.bin
│  ├── rustsbi-k210.bin
│  └── rustsbi-qemu.bin
├── docs                  # 文档
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
│     ├── boards          # 板卡信息
│     │  ├── mod.rs
│     │  └── qemu.rs
│     ├── config.rs       # qmeu平台配置
│     ├── driver          # 外设配置参数
│     │  ├── mod.rs
│     │  └── virt
│     │     └── mod.rs
│     ├── fat32           # FAT32文件系统
│     │  ├── mod.rs
│     │  └── virt.rs
│     ├── fs              #文件系统
│     │  ├── inode.rs
│     │  └── mod.rs
│     ├── lang_items.rs
│     ├── linker-qemu.ld
│     ├── logging.rs
│     ├── main.rs
│     ├── mm               #内存管理
│     │  ├── address.rs    # 虚拟/物理地址相关
│     │  ├── frame.rs  	   # 物理页帧分配
│     │  ├── heap.rs       # 堆分配
│     │  ├── mod.rs 
│     │  └── page.rs 	   # 页分配
│     ├── sbi              # SBI相关调用
│     │  ├── console.rs
│     │  ├── mod.rs
│     │  ├── qemu.rs
│     │  └── system.rs
│     ├── stack_trace.rs
│     ├── stdio.rs
│     ├── sync             # 同步模块
│     │  ├── mod.rs
│     │  └── up.rs
│     ├── syscall          # 系统调用
│     │  ├── fs.rs
│     │  ├── mod.rs
│     │  ├── process.rs
│     │  └── system.rs
│     ├── task             # 进程
│     │  ├── context.rs    # 进程上下文
│     │  ├── mod.rs
│     │  ├── pid.rs        # 进程pid
│     │  ├── processor.rs
│     │  ├── switch.rs
│     │  ├── switch.S
│     │  ├── task.rs
│     │  └── TaskManager.rs
│     ├── timer.rs
│     └── trap            # 异常处理
│        ├── context.rs   # 异常上下文处理
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
   └── src                # 测试脚本
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
