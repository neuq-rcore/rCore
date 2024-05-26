# 文件系统

文件系统相关模块用于读取 `sdcard.img` 镜像文件中的程序并执行

## 相关模块

- `fs`: 文件系统入口的包装
  - `inode`: 文件索引，包括对文件和文件夹的管理

## 模块 `fs`

`Rootfs` 结构体的函数如下：

- **new(raw_fs: FileSystem<Fat32IO>)**: 用来创建自身实例，在同模块中的懒静态引用使用：

```rust
lazy_static! {
    pub static ref ROOT_FS: RootFs = {
        let fs = Fat32FileSystem::new(0);
        debug!("Filesystem initialized.");
        RootFs::new(fs)
    };
}
```

单例模式的设计使得全局只存在一个 RootFs 实例。

- **root_dir(&'static self)**: 用来获取根目录，是后续读取文件的基础。

## 子模块 `inode`

### 自定义类型

为了增加可读性，本模块自定义了三种类型，后续不再解释：

```rust
pub type FatfsDir<'a> = Dir<'a, Fat32IO, NullTimeProvider, LossyOemCpConverter>;
pub type FatfsFile<'a> = File<'a, Fat32IO, NullTimeProvider, LossyOemCpConverter>;
pub type FatfsEntry<'a> = fatfs::DirEntry<'a, Fat32IO, NullTimeProvider, LossyOemCpConverter>;
```

主要内容是两个 `Fat32File` 和 `Fat32Dir` 结构体：

- `Fat32File` 是对 `FatfsEntry` 的包装，原生的函数如下：

  - **from_entry(entry: FatfsEntry<'a>)**: 从 `FatfsEntry` 构造对象。

  - **len(&self)**: 获取文件的长度。

  - **name(&self)**: 获取文件名。

  - **inner(&self)**: 获取 `FatfsEntry` 类型转换为 `FatfsFile` 的结果，用于更加底层的行为。

- `Fat32Dir` 是对枚举类型 `Fat32DirInner` 的包装，其定义如下：

```rust
enum Fat32DirInner<'a> {
    Root(FatfsDir<'a>),
    Sub(FatfsEntry<'a>),
}
```

- `Fat32DirInner` 的函数如下：

  - **from_root(root: FatfsDir<'a>)**: 从 `Fat32Dir` 构造对象。

  - **from_entry(entry: FatfsEntry<'a>)**: 从 `FatfsEntry` 构造对象。

  - **as_dir(&self)**: 返回转换为 `FatfsDir` 的结果。

- `Fat32Dir` 的函数如下：

  - **from_root(root: FatfsDir<'a>)**: 从 `Fat32Dir` 构造对象。

  - **from_entry(entry: FatfsEntry<'a>)**: 从 `Fat32Entry` 构造对象。

  - **inner(&self)**: 获取 `Fat32DirInner` 类型转换为 `FatfsDir` 的结果。

  - **name(&self)**: 获取文件名，如果是根目录则为 **None** 。

  - **match_dir(&self, name: &str)**: 私有函数，根据名字查找子目录。

  - **match_file(&self, name: &str)**: 私有函数，根据名字查找子文件。

  - **get_dir(&self, path: &str)**: 根据路径查找目录。

  - **get_file(&self, path: &str)**: 根据路径查找文件。

  - **read_file_as_buf(&self, path: &str)**: 根据路径查找文件，读取至内存并返回，后续可以被 `task` 模块处理。

## 使用方式

在实际使用环境中，通过如下代码执行一个程序：

```rust
let path = "path/to/your_program"
let buf = ROOT_FS.root_dir().read_file_as_buf(path);

match buf {
  Some(buf) => {
    task::kernel_create_process(&buf);
    task::run_tasks();
  }
  None => {
    info!("Program '{}' not found!", name);
  }
}
```