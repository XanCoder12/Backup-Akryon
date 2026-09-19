# Prerequisites & Toolchain Setup

Building and running Nyxara OS requires cross-compilation tools for 32-bit x86 targets, the freestanding Rust compiler toolchain, and the QEMU system emulator.

## Required Packages

- **NASM** (`nasm`): Netwide Assembler for MBR and 32-bit entry stubs.
- **GCC Multilib** (`gcc`, `gcc-multilib`): GNU C Compiler with 32-bit code generation (`-m32`).
- **GNU Binutils** (`ld`, `objcopy`): Linker and binary conversion utilities.
- **GNU Make** (`make`): Build orchestration.
- **Rust Toolchain** (`rustc`, `cargo`): Modern freestanding Rust compiler.
- **QEMU Emulator** (`qemu-system-i386`): Full system x86 PC emulator with VGA and RTL8139 emulation.
- **GDB** (`gdb`): GNU Debugger for source-level kernel inspection.

---

## Installation by Distribution

### Arch Linux / Manjaro
```bash
sudo pacman -S nasm gcc binutils qemu-system-x86 make rustup gdb
rustup default stable
rustup target add i686-unknown-linux-gnu
```

### Debian / Ubuntu / Linux Mint
```bash
sudo apt update
sudo apt install -y nasm gcc-multilib binutils qemu-system-x86 make rustc cargo gdb
# If using rustup:
rustup target add i686-unknown-linux-gnu
```

### Fedora / RHEL
```bash
sudo dnf install -y nasm gcc glibc-devel.i686 binutils qemu-system-x86 make rust cargo gdb
rustup target add i686-unknown-linux-gnu
```

---

## Verifying the Rust 32-bit Target

To confirm that the 32-bit x86 target is installed:

```bash
rustup target list | grep "i686-unknown-linux-gnu (installed)"
```

If missing, run:
```bash
rustup target add i686-unknown-linux-gnu
```

## Verifying the Build Environment

After installing all prerequisites, verify the complete build from the workspace root:

```bash
make clean
make
```

A successful build will output:
```
>>> Nyxara OS Image successfully built: nyxara.img (1.44 MB) <<<
```
