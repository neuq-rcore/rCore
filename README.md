# neuqOS

A simple os kernel for riscv64

## Build Dependencies

- `riscv64-elf-gcc`
- `riscv64-elf-binutils`
- `cargo-binutils`
- `llvm-tools-preview`

For command line instructions, refer to `.github/workflows/ci.yml`

## Build

```shell
# or simply run `make`
make build
```

## Run

```shell
make run
```

## Debug

### Command line

#### Launch QEMU instance with GDB server
```shell
make debug
```

#### Connect to GDB server
```shell
make connect
```

### VSCode
Open the repository in VSCode and press <kbd>F5</kbd> to start debugging.

## License

MIT
