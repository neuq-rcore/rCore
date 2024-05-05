#!/bin/bash

tmpdir=$(mktemp -d -p /tmp/)
qemu="qemu"
branch="stable-7.2"
repo=$(pwd)

echo "[!] Section: Setup ${qemu}"

echo "[!] Entering ${tmpdir}"
cd "$tmpdir" || exit 1

echo "[!] Fetching Qemu source code"
git clone https://github.com/qemu/qemu.git --depth=1 -b $branch $qemu

echo "[!] Entering ${qemu}"
cd "$qemu" || exit 1

echo "[!] Compiling Qemu"
./configure --target-list=riscv64-softmmu,riscv64-linux-user
make -j"$(nproc)"
sudo make install

echo "[!] Return to ${repo}"
cd "$repo" || exit 1