// Begin Region - Console
pub trait IConsole {
    fn init(&mut self);

    fn getchar(&self) -> u8;

    fn putchar(&mut self, c: u8);

    fn write(&mut self, str: &[u8]) {
        for &c in str.iter() {
            self.putchar(c);
        }
    }

    #[inline(always)]
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.write(s.as_bytes());

        Ok(())
    }
}

impl core::fmt::Write for dyn IConsole {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.write_str(s)
    }
}

// End Region - Console

use core::fmt::{Arguments, Write};
use core::panic;

// Begin Region - LegacyConsole
#[derive(Clone, Copy)]
pub struct LegacyConsole;

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

// End Region - LegacyConsole

// Begin Region - DebugConsole
use sbi_rt;
use sbi_spec::binary::Physical;
use sbi_spec::binary::{RET_ERR_DENIED, RET_ERR_FAILED, RET_ERR_INVALID_PARAM, RET_SUCCESS};

#[derive(Clone, Copy)]
pub struct DebugConsole;

static mut IS_DBCN_AVALIABLE: Option<bool> = None;

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

impl DebugConsole {
    pub fn get_api() -> Self {
        let mut con = Self;

        con.init();

        con
    }

    pub fn is_avaliable() -> bool {
        if let Some(ava) = unsafe { IS_DBCN_AVALIABLE } {
            return ava;
        }

        let is_avaliable = sbi_rt::probe_extension(sbi_rt::Console).is_available();
        unsafe {
            IS_DBCN_AVALIABLE = Some(is_avaliable);
        }
        is_avaliable
    }

    // Result<(char, count_read), err_code>
    fn sgetchar(&self) -> Result<(u8, usize), usize> {
        let c: u8 = 0;
        let p = Physical::new(1, &c as *const u8 as usize, 0);

        let ret = sbi_rt::console_read(p);

        match ret.error {
            RET_SUCCESS => Ok((c, ret.value)),
            _ => Err(ret.error),
        }
    }

    fn sputchar(&mut self, c: u8) -> Result<(), usize> {
        match sbi_rt::console_write_byte(c).error {
            RET_SUCCESS => Ok(()),
            err => Err(err),
        }
    }

    fn swrite(&mut self, str: &[u8]) -> Result<(), usize> {
        let p = Physical::new(str.len(), str.as_ptr() as usize, 0);

        match sbi_rt::console_write(p).error {
            RET_SUCCESS => Ok(()),
            err => Err(err),
        }
    }
}

impl core::fmt::Write for DebugConsole {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.swrite(s.as_bytes()).map_err(|_| core::fmt::Error)
    }
}

// End Region - DebugConsole

// Begin Region - UnionConsole
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

// End Region - LegacyConsole
