# 使用 SBI 的串口输出

本项目同时支持 Legacy console API 和 Debug console API 两种串口输出方式。

## IConsole Interface

本项目使用 `IConsole` 接口规定了串口输出的标准，其定义如下：

```rust
trait IConsole {
    fn init(&mut self);

    fn getchar(&self) -> u8;

    fn putchar(&mut self, c: u8);

    fn write(&mut self, str: &[u8]) {
        for &c in str.iter() {
            self.putchar(c);
        }
    }

    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.write(s.as_bytes());

        Ok(())
    }

}
```

- `init` 函数用于初始化串口。
- `putchar` 函数用于向串口输出一个字符。
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.write(s.as_bytes());

        Ok(())
    }

- `getchar` 函数用于从串口读取一个字符。
- `write` 函数用于向串口输出一个字符切片（`&[u8]`）。
- `write_str` 函数用于向串口输出一个字符串。

## LegacyConsole

`LegacyConsole` 使用 SBI 的 Legacy console API 进行串口输出，其实现如下：

```rust
impl LegacyConsole {
    pub fn get_api() -> Self {
        // no need to init it
        Self
    }
}

impl IConsole for LegacyConsole {
    fn init(&mut self) {}

    #[allow(deprecated)]
    fn getchar(&self) -> u8 {
        sbi_rt::legacy::console_getchar() as u8
    }

    #[allow(deprecated)]
    fn putchar(&mut self, c: u8) {
        sbi_rt::legacy::console_putchar(c as usize);
    }
}

impl core::fmt::Write for LegacyConsole {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        IConsole::write_str(self, s)
    }
}
```

Legacy Console API 每次只支持一个字符的输入输出，所以 `getchar` 和 `putchar` 函数只能处理一个字符。通过`IConsole`的默认实现，可以支持对字符串的输出以及格式化输出。

## DebugConsole

`DebugConsole` 使用 SBI 的 Debug console API 进行串口输出，其实现如下：

```rust
impl IConsole for DebugConsole {
    fn init(&mut self) {
        if !Self::is_avaliable() {
            panic!("Debug Console Extension is not avaliable. Please update your SBI to specification v2.0 and ensure DBCN is enabled");
        }
    }

    fn getchar(&self) -> u8 {
        match self.sgetchar() {
            Ok((c, 1)) => c,
            Ok((_, 0)) => 0,
            Ok((c, cnt)) => {
                panic!(
                    "Expect to read 1 char, but got {0}, first char: {1}",
                    cnt, c
                );
            }
            Err(err) => {
                match err {
                    RET_ERR_INVALID_PARAM => panic!("The memory pointed to by the \'num_bytes\', \'base_addr_lo\', and \'base_addr_hi\' parameters does not satisfy the requirements described in the \'shared_memory_physical_address_range_parameter\'"),
                    RET_ERR_DENIED => panic!("Reads from the debug console is not allowed"),
                    RET_ERR_FAILED => panic!("Failed to read due to I/O errors"),
                    _ => panic!("Unknown error occurred when trying to read bytes from debug console, error code: {0}", err),
                };
            }
        }
    }

    fn putchar(&mut self, c: u8) {
        if let Err(err) = self.sputchar(c) {
            match err {
                RET_ERR_DENIED => panic!("Write to the debug console is not allowed"),
                RET_ERR_FAILED => panic!("Failed to write the byte due to I/O errors"),
                _ => panic!("Unknown error occurred when trying to write byte from debug console, char: {0}, code: {1}", c, err),
            }
        }
    }

    fn write(&mut self, str: &[u8]) {
        if let Err(err) = self.swrite(str) {
            match err {
                RET_ERR_INVALID_PARAM => panic!("The memory pointed to by the \'num_bytes\', \'base_addr_lo\', and \'base_addr_hi\' parameters does not satisfy the requirements described in the \'shared_memory_physical_address_range_parameter\'"),
                RET_ERR_DENIED => panic!("Writes to the debug console is not allowed"),
                RET_ERR_FAILED => panic!("Failed to write due to I/O errors."),
                _ => panic!("Unknown error occurred when trying to write bytes to debug console, error code: {0}", err),
            }
        }
    }
}
```

Debug Console API 支持一次读取多个字符，所以 `getchar` 函数可以读取多个字符。`putchar` 函数也支持一次输出多个字符。

Debug Console 一次性向 SBI 发送整个字符串，减少内核在S态和M态之间的切换次数，提高性能。但是 Debug Console API 需要 SBI v2.0 的支持，所以需要在初始化时检查 SBI 的版本。

## UnionConsole

`UnionConsole` 是对 `LegacyConsole` 和 `DebugConsole` 的封装，可以同时支持两种串口输出方式。其实现如下：

```rust
#[derive(Clone, Copy)]
pub enum UnionConsole {
    Legacy(LegacyConsole),
    Dbcn(DebugConsole),
}

impl UnionConsole {
    fn new() -> UnionConsole {
        match DebugConsole::is_avaliable() {
            true => Self::new_dbcn(),
            false => Self::new_legacy(),
        }
    }

    fn new_legacy() -> UnionConsole {
        UnionConsole::Legacy(LegacyConsole::get_api())
    }

    fn new_dbcn() -> UnionConsole {
        UnionConsole::Dbcn(DebugConsole::get_api())
    }

    #[allow(unused)]
    fn read(&self) -> u8 {
        match self {
            UnionConsole::Legacy(leg) => leg.getchar(),
            UnionConsole::Dbcn(dbcn) => dbcn.getchar(),
        }
    }

    #[allow(unused)]
    fn put(&mut self, c: u8) {
        match self {
            UnionConsole::Legacy(leg) => leg.putchar(c),
            UnionConsole::Dbcn(dbcn) => dbcn.putchar(c),
        }
    }

    pub fn write_fmt(&mut self, arg: Arguments) {
        match self {
            UnionConsole::Legacy(leg) => leg.write_fmt(arg).unwrap(),
            UnionConsole::Dbcn(dbcn) => dbcn.write_fmt(arg).unwrap(),
        }
    }
}

static mut GLOBAL_CONSOLE: Option<UnionConsole> = None;

impl UnionConsole {
    pub fn instance() -> UnionConsole {
        match unsafe { GLOBAL_CONSOLE } {
            Some(console) => console,
            None => {
                let instance = UnionConsole::new();
                unsafe { GLOBAL_CONSOLE = Some(instance) };
                instance
            }
        }
    }

    #[allow(unused)]
    pub fn force_assign(instance: UnionConsole) {
        unsafe {
            GLOBAL_CONSOLE = Some(instance);
        }
    }

    #[allow(unused)]
    pub fn getchar() -> u8 {
        Self::instance().read()
    }

    #[allow(unused)]
    pub fn putchar(c: u8) {
        Self::instance().put(c)
    }

    #[allow(unused)]
    pub fn printf(arg: Arguments) {
        Self::instance().write_fmt(arg)
    }
}
```

通过 lazy evaluation 的方式，`UnionConsole` 可以根据 SBI 的版本自动选择合适的串口输出方式。同时，`UnionConsole` 也支持多种串口输出方式，可以同时支持两种串口输出方式。

## println! 和 print! 宏

`println!`和`print!`宏是对`UnionConsole`的封装，可以方便地进行串口输出。其实现如下：

```rust
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {{
        write!($crate::sbi::console::UnionConsole::instance(), $($arg)*);
    }};
}

#[macro_export]
macro_rules! println {
    () => {
        writeln!($crate::sbi::console::UnionConsole::instance());
    };
    ($($arg:tt)*) => {{
        writeln!($crate::sbi::console::UnionConsole::instance(), $($arg)*);
    }};
}
```