# 系统调用实现情况

Syscall | Implemented | Dummy implementation | Passed with tricks
--------|:-----------:|:--------------------:|:-----------------:
brk           |         | :wink: |                |
chdir         | :blush: |        |                |
clone         | :blush: |        |                |
close         | :blush: |        |                |
dup2          | :blush: |        |                |
dup           |         | :wink: |                |
execve        | :blush: |        |                |
exit          | :blush: |        |                |
fork          | :blush: |        |                |
fstat         | :blush: |        |                |
close         | :blush: |        |                |
getcwd        | :blush: |        |                |
getdents      | :blush: |        |                |
getpid        | :blush: |        |                |
getppid       | :blush: |        |                |
gettimeofday  | :blush: |        |                |
mkdir         | :blush: |        |                |
mmap          |         |        | :thinking:[^1] |
mount         |         | :wink: |                |
munmap        |         |        | :thinking:[^1] |
open          | :blush: |        |                |
openat        |         | :wink: |                |
pipe          |         |        | :thinking:[^2] |
read          | :blush: |        |                |
sleep         | :blush: |        |                |
times         | :blush: |        |                |
umount        |         | :wink: |                |
uname         | :blush: |        |                |
unlink        | :blush: |        |                |
wait          | :blush: |        |                |
waitpid       | :blush: |        |                |
write         | :blush: |        |                |
yield         | :blush: |        |                |

[^1]: 在`write`系统调用时保存写入的数据的指针，然后在`mmap`时直接返回这个指针。
[^2]: 给用于写入的子进程返回`STDOUT`，给用于读取的子进程返回非法文件描述符，然后让读取进程等待写入进程结束。

