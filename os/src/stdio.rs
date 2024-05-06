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
