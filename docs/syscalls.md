#系统调用实现情况

Syscall | Implemented | Dummy implementation | Passed with tricks
--------|-------------|----------------------|-----------
brk           |    | 😉 |    |
chdir         | 😀 |    |    |
clone         | 😀 |    |    |
close         | 😀 |    |    |
dup2          |    |    | 🤔[^1] |
dup           |    | 😉 |    |
execve        | 😀 |    |    |
exit          | 😀 |    |    |
fork          | 😀 |    |    |
fstat         | 😀 |    |    |
close         | 😀 |    |    |
getcwd        | 😀 |    |    |
getdents      | 😀 | 😉 |    |
getpid        | 😀 |    |    |
getppid       | 😀 |    |    |
gettimeofday  | 😀 |    |    |
mkdir         | 😀 |    |    |
mmap          |    |    | 🤔[^2] |
mount         |    | 😉 |    |
munmap        |    |    | 🤔[^2] |
open          | 😀 |    |    |
openat        |    | 😉 |    |
pipe          |    | 😉 |    |
read          | 😀 |    |    |
sleep         | 😀 |    |    |
times         | 😀 |    |    |
umount        |    | 😉 |    |
uname         | 😀 |    |    |
unlink        | 😀 |    |    |
wait          | 😀 |    |    |
waitpid       | 😀 |    |    |
write         | 😀 |    |    |
yield         | 😀 |    |    |

[^1]: 给用于写入的子进程返回`STDOUT`，给用于读取的子进程返回非法文件描述符，然后让读取进程等待写入进程结束。
[^2]: 在`write`系统调用时保存写入的数据的指针，然后在`mmap`时直接返回这个指针。

