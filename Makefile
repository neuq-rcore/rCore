.PHONY: clippy clippy-% all test test-inner parse dummy-run dummy-test vf2

all:
	@cd os && make -s release

	@cp os/target/riscv64gc-unknown-none-elf/release/os.bin kernel-qemu
	@cp bootloader/opensbi-qemu.bin sbi-qemu

clippy: clippy-os

clippy-%:
	cd $* && cargo clippy --all-features

test: all test-inner parse

test-inner:
	@cp test/sdcard.img .

	@qemu-system-riscv64 -machine virt \
        -m 128M -nographic -smp 2 \
        -kernel kernel-qemu \
        -bios sbi-qemu \
        -drive file=sdcard.img,if=none,format=raw,id=x0 \
        -device virtio-blk-device,drive=x0,bus=virtio-mmio-bus.0 \
        -device virtio-net-device,netdev=net \
        -netdev user,id=net | tee output.log

parse:
# the test scripts produce 'SyntaxWarning: invalid escape sequence'
	@python3 -W ignore test/check_result/test_runner.py output.log > results.json
	@python3 test/visualize_result.py results.json || exit 0

dummy-run:
	@python3 test/demo_test.py test/riscv64/ | tee output.log

dummy-test: dummy-run parse

clean:
	@echo -e "\e[1;33m=> Cleaning os...\e[0m"
	@cd os && make -s clean
	@echo -e "\e[1;33m=> Cleaning all outputs/dependencies...\e[0m"
	rm -f sbi-qemu kernel-qemu
	@echo -e "\e[1;33m=> Cleaning test results\e[0m"
	rm -f output.log results.json sdcard.img
	@echo -e "\e[1;33m=> Cleaning test result parse\e[0m"
	rm -rf test/check_result/__pycache__

vf2: 
	@cd os && make -s release-vf2

%:
	@cd os && make -s $@
