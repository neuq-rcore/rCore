const SYSCALL_WRITE: usize = 64;
const SYSCALL_EXIT: usize = 93;
const SYSCALL_YIELD: usize = 124;
const SYSCALL_GET_TIME: usize = 169;
const SYSCALL_NANOSLEEP: usize = 101;
const SYSCALL_UNAME: usize = 160;
const SYSCALL_GETPID: usize = 172;
const SYSCALL_GETPPID: usize = 173;
const SYSCALL_TIMES: usize = 153;
const SYSCALL_CLONE: usize = 220;
const SYSCALL_EXEC: usize = 221;
const SYSCALL_GETCWD: usize = 17;
const SYSCALL_WAIT: usize = 260;
const SYSCALL_MOUNT: usize = 40;
const SYSCALL_UNMOUNT: usize = 39;
const SYSCALL_BRK: usize = 214;
const SYSCALL_DUP: usize = 23;
const SYSCALL_DUP2: usize = 24;
const SYSCALL_CHDIR: usize = 49;
const SYSCALL_MKDIR: usize = 34;
const SYSCALL_OPENAT: usize = 56;
const SYSCALL_CLOSE: usize = 57;
const SYSCALL_READ: usize = 63;
const SYSCALL_PIPE: usize = 59;
const SYSCALL_FSTAT: usize = 80;
const SYSCALL_UNLINKAT: usize = 35;
const SYSCALL_GETDENTS: usize = 61;
const FREEDOM_DIVE: usize = 222;
const SYSCALL_MUNMAP: usize = 215;

mod fs;
mod process;
mod system;

use fs::*;
use log::debug;
use log::warn;
use process::*;
use system::*;

use crate::timer::TimeVal;

pub fn syscall(syscall_id: usize, args: [usize; 6]) -> isize {
    debug!("Syscall received, id: {}", syscall_id);

    let ret = match syscall_id {
        SYSCALL_WRITE => sys_write(args[0], args[1] as *const u8, args[2]),
        SYSCALL_EXIT => sys_exit(args[0] as i32),
        SYSCALL_YIELD => sys_yield(),
        SYSCALL_TIMES => sys_times(args[0]),
        SYSCALL_GET_TIME => sys_get_time(args[0] as *mut TimeVal, args[1] as i32),
        SYSCALL_NANOSLEEP => sys_nanosleep(args[0] as *mut TimeVal, args[1] as *mut TimeVal),
        SYSCALL_UNAME => sys_uname(args[0] as *mut Utsname),
        SYSCALL_GETPID => sys_getpid(),
        SYSCALL_GETPPID => sys_getppid(),
        SYSCALL_EXEC => sys_exec(
            args[0] as *const u8,
            args[1] as *const *const u8,
            args[2] as *const *const u8,
        ),
        SYSCALL_CLONE => sys_clone(args[0], args[1], args[2]),
        SYSCALL_WAIT => sys_waitpid(args[0] as isize, args[1] as *mut isize, args[2]),
        SYSCALL_MOUNT | SYSCALL_UNMOUNT => 0,
        SYSCALL_GETCWD => sys_getcwd(args[0] as *mut u8, args[1]),
        SYSCALL_BRK => sys_brk(args[0]),
        SYSCALL_DUP => 0,
        SYSCALL_DUP2 => sys_dup2(args[0] as isize, args[1] as isize),
        SYSCALL_CHDIR => sys_chdir(args[0] as *const u8),
        SYSCALL_MKDIR => sys_mkdirat(args[0] as isize, args[1] as *const u8, args[2] as u32),
        SYSCALL_OPENAT => sys_openat(args[0] as isize, args[1] as *const u8, args[2] as u32),
        SYSCALL_CLOSE => sys_close(args[0] as usize),
        SYSCALL_READ => sys_read(args[0] as usize, args[1] as *mut u8, args[2]),
        SYSCALL_PIPE => sys_pipe(args[0] as *mut (i32, i32)),
        SYSCALL_FSTAT => sys_fstat(args[0] as usize, args[1] as *mut u8),
        SYSCALL_UNLINKAT => sys_unlinkat(args[0] as isize, args[1] as *const u8, args[2] as u32),
        SYSCALL_GETDENTS => sys_getdents(args[0] as isize, args[1] as *mut u8, args[2]),
        FREEDOM_DIVE => sys_mmap(args[4] as isize),
        SYSCALL_MUNMAP => sys_munmap(args[0]),
        _ => {
            warn!("Unsupported syscall: {}, kernel killed it.", syscall_id);
            sys_exit(-1);
        }
    };

    debug!("Syscall returned: {}", ret);

    ret
}
