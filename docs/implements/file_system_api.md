# 文件系统 API

文件系统 API 为使用者管理和操作文件系统树（File System Tree）提供了方便的 API 接口。

## 相关依赖

- `fatfs`

## 相关模块

- `fs`: 文件系统入口的包装，和文件夹和文件的结构体的包装
  - `inode`: 文件索引，管理文件的权限

## 模块 `fs`

为了增加可读性，本模块自定义了三种类型，后续不再解释：

```rust
pub type FatfsDir<'a> = Dir<'a, Fat32IO, NullTimeProvider, LossyOemCpConverter>;
pub type FatfsFile<'a> = File<'a, Fat32IO, NullTimeProvider, LossyOemCpConverter>;
pub type FatfsEntry<'a> = fatfs::DirEntry<'a, Fat32IO, NullTimeProvider, LossyOemCpConverter>;
```

`Rootfs` 结构体是对 `sdcard.img` 的抽象，函数如下：

- **new(raw_fs: FileSystem<Fat32IO>)**: 用来创建自身实例，在同模块中的懒静态引用使用：

```rust
lazy_static! {
    pub static ref ROOT_FS: Arc<RootFs> = Arc::new(RootFs::new(0));
}

pub fn get_fs() -> Arc<RootFs> {
    ROOT_FS.clone()
}
```

单例模式的设计使得全局只存在一个 RootFs 实例，通过 `get_fs` 函数获得，保证多线程并发时的安全。

- **root_dir(&'static self)**: 用来获取根目录，是后续读取文件的基础。

`Fat32File` 和 `Fat32Dir` 这两个结构体分别是文件和目录的抽象。

- `Fat32File` 是对 `FatfsEntry` 的包装，原生的函数如下：

  - **from_entry(entry: FatfsEntry<'a>)**: 从 `FatfsEntry` 构造对象。

  - **len(&self)**: 获取文件的长度。

  - **name(&self)**: 获取文件名。

  - **as_file(&self)**: 将自身类型转换为 `FatfsFile` 并返回。

  - **as_entry(&self)**: 将自身类型转换为 `FatfsEntry` 并返回。

- `Fat32Dir` 是对枚举类型 `Fat32DirInner` 的包装，其定义如下：

```rust
enum Fat32DirInner<'a> {
    Root(FatfsDir<'a>), // 根目录
    Sub(FatfsEntry<'a>), // 除根目录以外的目录或文件
}
```

根目录是 `'/'` ，没有文件名，所以要单独分离。

- `Fat32Dir` 的函数如下：

  - **from_root(root: FatfsDir<'a>)**: 从 `Fat32Dir` 构造对象。

  - **from_entry(entry: FatfsEntry<'a>)**: 从 `FatfsEntry` 构造对象。

  - **as_dir(&self)**: 将自身类型转换为 `FatfsDir` 并返回。

  - **as_entry(&self)**: 将自身类型转换为 `FatfsEntry` 并返回。

  - **name(&self)**: 获取文件名，如果是根目录则为空。

  - **get_parent_dir(&self, path: &str)**: 获取父目录。

  - **get_dir(&self, path: &str)**: 根据路径查找目录。

  - **get_file(&self, path: &str)**: 根据路径查找文件。

  - **read_file_as_buf(&self, path: &str)**: 根据路径查找文件，读取至内存并返回，后续可以被 `task` 模块处理。

## 使用方式

在实际使用环境中，通过如下代码执行指定目录下的一个程序：

```rust
let path = "/path/to/your_program"
let buf = get_fs().root_dir().read_file_as_buf(path);

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