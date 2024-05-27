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
const SYSCALL_OPEN: usize = 56;
const SYSCALL_CLOSE: usize = 57;

mod fs;
mod process;
mod system;

use log::warn;
use log::debug;
use fs::*;
use process::*;
use system::*;

use crate::timer::TimeVal;

pub fn syscall(syscall_id: usize, args: [usize; 3]) -> isize {
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
        SYSCALL_EXEC => sys_exec(args[0] as *const u8, args[1] as *const *const u8, args[2] as *const *const u8),
        SYSCALL_CLONE => sys_clone(args[0], args[1], args[2]),
        SYSCALL_WAIT => sys_waitpid(args[0] as isize, args[1] as *mut isize, args[2]),
        SYSCALL_MOUNT | SYSCALL_UNMOUNT => 0,
        SYSCALL_GETCWD => sys_getcwd(args[0] as *mut u8, args[1]),
        SYSCALL_CHDIR => sys_chdir(args[0] as *const u8),
        SYSCALL_BRK => sys_brk(args[0]),
        SYSCALL_DUP => 0,
        SYSCALL_DUP2 => sys_dup2(args[0] as isize, args[1] as isize),
        SYSCALL_CHDIR => sys_chdir(args[0] as *const u8),
        SYSCALL_MKDIR => 0,
        _ => {
            warn!("Unsupported syscall: {}, kernel killed it.", syscall_id);
            sys_exit(-1);
        }
    };

    debug!("Syscall returned: {}", ret);

    ret
}
