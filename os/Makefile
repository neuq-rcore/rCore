# Region // Build
OUTPUT_NAME = os

CARGO := cargo
CARGO_FLAGS :=
MODE := debug
TARGET := riscv64gc-unknown-none-elf
KERNEL_ELF := target/$(TARGET)/$(MODE)/$(OUTPUT_NAME)
KERNEL_BIN := $(KERNEL_ELF).bin

OBJCOPY := rust-objcopy --binary-architecture=riscv64

ifeq ($(MODE), release)
	CARGO_FLAGS += --release
endif

# End Region // Build

# Region // Debugging
ARCH := riscv64
GDB := $(ARCH)-elf-gdb
# End Region // Debugging

# Region // Board
QEMU := qemu-system-$(ARCH)

# alternative : 'opensbi' and 'rustsbi' 
SBI_IMPL := opensbi
BOARD_TYPE := qemu
BOOTLOADER := ../bootloader/$(SBI_IMPL)-$(BOARD_TYPE).bin

KERNEL_ENTRY_PA := 0x80200000

QEMU_ARGS := -machine virt \
			 -nographic \
			 -bios $(BOOTLOADER) \
			 -device loader,file=$(KERNEL_ELF),addr=$(KERNEL_ENTRY_PA) \
			 -drive file=../sdcard.img,if=none,format=raw,id=x0 \
        	 -device virtio-blk-device,drive=x0,bus=virtio-mmio-bus.0
# End Region // Board

all: build

build:
	@echo '[+] Building os kernel'
	$(CARGO) build $(CARGO_FLAGS)
	@$(OBJCOPY) --strip-all $(KERNEL_ELF) $(KERNEL_BIN)

run: build prepare-sdcard
	@$(QEMU) $(QEMU_ARGS)

debug: build prepare-sdcard
	@echo '[!] Waiting for debugger'
	@$(QEMU) $(QEMU_ARGS) -s -S

prepare-sdcard:
	cp ../test/sdcard.img ../sdcard.img

gdb: connect

connect:
	@$(GDB) -ex 'file $(KERNEL_ELF)' \
		-ex 'set arch riscv:rv64' \
		-ex 'target remote localhost:1234' \
		-ex 'layout src'

help:
	@echo '- make build   : Build os kernel'
	@echo '- make run     : Run os kernel with qemu'
	@echo '- make debug   : Launch the os with qemu and wait for gdb'
	@echo '- make connect : Launch gdb and connect to qemu'

release: CARGO_FLAGS += --release
release: MODE = release
release: KERNEL_ELF = target/$(TARGET)/$(MODE)/$(OUTPUT_NAME)
release: KERNEL_BIN = $(KERNEL_ELF).bin
release: release-pre release-inner

release-pre:
	@echo 'Preparing for release build...'

release-inner:
	@echo '[+] Building os kernel(release)'
	$(CARGO) build $(CARGO_FLAGS)
	@$(OBJCOPY) --strip-all $(KERNEL_ELF) $(KERNEL_BIN)

release-vf2: CARGO_FLAGS += --features board_vf2
release-vf2: release

clean:
	@$(CARGO) clean
