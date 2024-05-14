import os
import sys
import subprocess

test_binaries = ["brk", "chdir", "clone", "close", "dup2", "dup", "execve", "exit", "fork", "fstat", "getcwd", "getdents", "getpid", "getppid", "gettimeofday", "mkdir", "mmap", "mount", "munmap", "open", "openat", "pipe", "read", "sleep", "times", "umount", "uname", "unlink", "wait", "waitpid", "write", "yield"]
qemu = "qemu-riscv64"
rootfs = None

def run_test(binary: str, name: str):
    # add `[KERNEL]` prefix to assume that the output is from the kernel
    print(f"[KERNEL] Running test {binary}")
    command = [qemu, binary]

    if rootfs is not None:
        command += ["-L", rootfs]

    # wait until the process is finished
    try:
        result = subprocess.run(command, stdout=subprocess.PIPE)
        print(result.stdout.decode("utf-8"))
    except subprocess.CalledProcessError as e:
        print(e.output.decode("utf-8"))

        # Add end mark to make the script easier to parse
        print(f"\n========== END test_{name} ==========")
        print(f"[KERNEL] Exit code {e.returncode} for test {name}")

def main(path: str):
    print(r"                         _____ ____ _____ ")
    print(r"                        / ____|  _ \_   _|")
    print(r" _ __   ___ _   _  __ _| (___ | |_) || |  ")
    print(r"| '_ \ / _ \ | | |/ _` \\___ \|  _ < | |  ")
    print(r"| | | |  __/ |_| | (_| |____) | |_) || |_ ")
    print(r"|_| |_|\___|\__,_|\__, |_____/|____/_____|")
    print(r"                     |_|                  ")
    print(r"Platform Name             : Qemu-user-riscv64")
    print(r"Platform Features         : None")
    print(r"Platform HART Count       : 1")
    print(r"Firmware Base             : Unknown")
    print(r"Firmware Size             : Unknown")
    print(r"Runtime SBI Version       : Unknown")

    for test in test_binaries:
        binary_path = os.path.join(path, test)
        run_test(binary_path, test)

def detect_qemu():
    return os.path.exists(qemu)

if __name__ == "__main__":
    dir = sys.argv[1]

    if len(sys.argv) > 2:
        rootfs = sys.argv[2]

    main(dir)
