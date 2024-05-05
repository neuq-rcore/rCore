#!/bin/bash

tmpdir=$(mktemp -d -p /tmp/)
qemu="qemu-7.2.9"
repo=$(pwd)

echo "[!] Section: Setup ${qemu}"

echo "[!] Entering ${tmpdir}"
cd "$tmpdir" || exit 1

echo "[!] Fetching Qemu source code"
wget "https://download.qemu.org/${qemu}.tar.xz"

echo "[!] Extracting source code"
tar xvf "${qemu}.tar.xz"

echo "[!] Entering ${qemu}"
cd "$qemu" || exit 1

echo "[!] Compiling Qemu"
./configure --target-list=riscv64-softmmu,riscv64-linux-user
make -j"$(nproc)"
sudo make install

echo "[!] Return to ${repo}"
cd "$repo" || exit 1