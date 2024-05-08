.PHONY: clippy clippy-% all test

all:
	@echo -e '\e[31mNOT IMPLEMENTED!\e[0m'
	@exit 1

clippy: clippy-user clippy-os

clippy-%:
	cd $* && cargo clippy --all-features

test: all
	@cp test/sdcard.img .

	@qemu-system-riscv64 -machine virt \
        -m 128M -nographic -smp 2 \
        -kernel kernel-qemu \
        -bios sbi-qemu \
        -drive file=sdcard.img,if=none,format=raw,id=x0 \
        -device virtio-blk-device,drive=x0,bus=virtio-mmio-bus.0 \
        -device virtio-net-device,netdev=net \
        -netdev user,id=net | tee output.log

	@python3 test/check_result/test_runner.py output.log > results.json
	@python3 test/visualize_result.py results.json

%:
	@cd os && make -s $@