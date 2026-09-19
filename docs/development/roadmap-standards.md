# Standards, Workspace Roles & Roadmap

This document defines the engineering standards, directory responsibilities, and future architectural milestones for Nyxara OS.

## Strict Code Style Standards

Nyxara enforces a clean, minimalistic, and maintainable codebase as specified in `CLAUDE.md`:

1. **No AI Slop / Over-commenting**
   - Redundant or obvious comments are prohibited (e.g. repeating function names in docstrings or commenting lines like `// initialize x`).
   - Comments must be reserved exclusively for non-obvious hardware behaviors, register bitmasks, or complex low-level algorithms.

2. **No Heavy ASCII Separators**
   - Decorative dividers such as `// ==========================`, `/* ----------------- */`, or `##########` are strictly forbidden in new code.
   - Clean, standard spacing and concise markdown headers must be used instead.

3. **Code Conciseness & Language Policy**
   - **Rust**: `#![no_std]`, `#![no_main]`. Encapsulate hardware access in safe abstractions with explicit, minimal `unsafe` blocks.
   - **C**: Freestanding compilation (`-ffreestanding -nostdlib`). Use explicit fixed-width integers (`uint32_t`, `size_t`) from `hal/types.h`.
   - **Assembly**: NASM syntax with minimal boilerplate.

---

## Directory Roles & Workspace Architecture

The repository is divided into focused directories to separate responsibilities:

| Directory | Purpose & Status |
|---|---|
| **`boot/`** | Active. 16-bit MBR loader (`boot.asm`) and 32-bit protected mode entry (`kernel_entry.asm`). |
| **`hal/`** | Active. Hardware Abstraction Layer in C (GDT, IDT, PIC, PIT, UART, PCI, RTL8139, VGA/LFB, PS/2). |
| **`kernel/`** | Active. Kernel initialization (`kmain.c`) and C-to-Rust calling trampolines. |
| **`rust/`** | Active. Core Rust kernel (PMM, Heap, VMM Paging, Syscalls, VFS, Network Stack, Shell, Editor). |
| **`drivers/`** | Planned. Modular driver plugins built on top of HAL (sound, USB, AHCI). |
| **`libc/`** | Planned. Freestanding C runtime library and syscall wrappers for Ring 3 user programs. |
| **`userland/`** | Planned. Standalone user applications and services executing outside supervisor mode. |
| **`fs/`** | Planned. Persistent block storage filesystem drivers (ext2, FAT32). |
| **`include/`** | Planned. Shared header files exposed to drivers and userland. |
| **`tests/`** | Planned. Automated unit tests, integration tests, and emulator validation scripts. |
| **`tools/`** | Planned. Disk image packaging, asset generators, and development utilities. |
| **`config/`** | Target architecture and build configuration files. |

---

## Architectural Roadmap

### Phase 1: Preemptive Multitasking & Scheduler
- **Process Control Block (PCB)**: Structure holding CPU registers, process ID (PID), task state (Ready, Running, Blocked, Zombie), and page directory reference.
- **PIT Context Switch**: Hook IRQ 0 to perform preemptive round-robin thread switching by preserving and swapping stack frames.
- **Kernel Threads**: Enable background worker threads for network socket polling and asynchronous tasks.

### Phase 2: Ring 3 Userland Process Isolation
- **TSS Configuration**: Complete stack switching mechanism (`ss0`, `esp0`) when transitioning from Ring 3 to Ring 0.
- **User Memory Space**: Allocate isolated page directories for each process (`0x04000000..0xBFFFFFFF`) while sharing identity-mapped kernel supervisor pages.
- **Full POSIX Syscall Integration**: Expand `int 0x80` to support `sys_fork`, `sys_execve`, `sys_waitpid`, and `sys_brk` (heap expansion).

### Phase 3: ELF Binary Executable Loader
- Parse 32-bit ELF headers and program headers (`PT_LOAD`).
- Map ELF segments into user address space with appropriate page permissions (Executable vs Writable).
- Jump to the ELF entry point in Ring 3 via an `iret` frame.

### Phase 4: Persistent Block Storage & Filesystems
- Implement ATA/IDE PIO and AHCI disk controllers.
- Develop an ext2 and FAT32 filesystem driver in Rust, mounting onto the existing VFS node tree.
- Persist shell user files across reboots and QEMU sessions.
