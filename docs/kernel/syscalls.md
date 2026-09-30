# Unix System Calls (`int 0x80`)

Nyxara has a small Unix-like syscall dispatcher on software interrupt vector 128 (`0x80`). The gate is callable from Ring 3, but the syscall set is incomplete and does not yet provide POSIX compatibility or safe user-memory handling.

## ABI & Register Calling Convention

Nyxara adheres to the standard Linux i386 ABI calling convention:

| Register | Purpose | Description |
|---|---|---|
| **`EAX`** | Syscall Number / Return Value | Input: System call ID. Output: Return code or negative errno. |
| **`EBX`** | Argument 1 | First parameter (e.g. file descriptor or exit code) |
| **`ECX`** | Argument 2 | Second parameter (e.g. buffer pointer) |
| **`EDX`** | Argument 3 | Third parameter (e.g. buffer length) |
| **`ESI`** | Argument 4 | Fourth parameter |
| **`EDI`** | Argument 5 | Fifth parameter |

## Declared and Implemented Calls

```rust
pub const SYS_EXIT:    u32 = 1;
pub const SYS_FORK:    u32 = 2;
pub const SYS_READ:    u32 = 3;
pub const SYS_WRITE:   u32 = 4;
pub const SYS_OPEN:    u32 = 5;
pub const SYS_CLOSE:   u32 = 6;
pub const SYS_WAITPID: u32 = 7;
pub const SYS_EXECVE:  u32 = 11;
pub const SYS_GETPID:  u32 = 20;
```

Implemented: `SYS_EXIT`, `SYS_FORK`, `SYS_WAITPID`, `SYS_EXECVE`, `SYS_READ`, `SYS_WRITE`, and `SYS_GETPID`. `SYS_OPEN` and `SYS_CLOSE` fall through to `-ENOSYS` (`-38`).

`fork`, `waitpid`, `execve`, and `exit` are gated to Ring 3 (`regs.cs & 3 == 3`); calling them from kernel context returns `-EPERM`. The process semantics are described in [Userland](userland.md).

### System Call Descriptions

### `sys_exit` (`EAX = 1`)
- **Arguments**: `EBX`: Exit status code (`int status`).
- **Behavior**: Terminates the calling Ring 3 task: releases its address space, wakes a parent blocked in `waitpid` (delivering the status), and never returns. The task becomes a `Zombie` until reaped.

### `sys_fork` (`EAX = 2`)
- **Returns**: Child pid in the parent, `0` in the child.
- **Behavior**: Clones the caller — user pages are copied eagerly into a new page directory, and the child resumes with an identical register frame except `EAX = 0`. Returns `-EAGAIN` when the task table is full, `-ENOMEM` on page allocation failure.

### `sys_read` (`EAX = 3`)
- **Arguments**: `EBX`: File descriptor (`int fd`), `ECX`: Destination buffer pointer (`void* buf`), `EDX`: Maximum bytes to read (`size_t count`).
- **Current behavior**: Stub that returns `0`; it does not read stdin or files.

### `sys_write` (`EAX = 4`)
- **Arguments**: `EBX`: File descriptor (`int fd`), `ECX`: Source buffer pointer (`const void* buf`), `EDX`: Byte count (`size_t count`).
- **Behavior**:
  - `fd == 1` (stdout) or `fd == 2` (stderr): Writes characters directly to the VGA display and serial debug log.
  - Invalid descriptors return `-EBADF` (`-9`).

`sys_write` currently dereferences the supplied buffer directly. User pointers are not checked against mapped user pages, so this interface is not safe for untrusted user programs yet.

### `sys_waitpid` (`EAX = 7`)
- **Arguments**: `EBX`: Child pid or `-1` for any child, `ECX`: Status pointer (`int* status`) receiving the raw exit code.
- **Behavior**: Reaps an exited child and returns its pid; blocks the caller until a matching child exits when none has. `-ECHILD` (`-10`) when the pid is not a child.

### `sys_execve` (`EAX = 11`)
- **Arguments**: `EBX`: NUL-terminated path (`const char* path`); `ECX`/`EDX`: `argv`/`envp`, accepted but unused.
- **Behavior**: Loads an ELF32 image from the VFS (initrd programs live in `/bin`), replaces the caller's address space and register frame, and "returns" into the new program. Errors: `-EFAULT` (`-14`) for unmapped path pointers, `-ENOENT` (`-2`) when the path does not resolve, `-ENOEXEC` (`-8`) for invalid images.

The path string is read safely: every byte's page is translated through the active page directory before access, so unmapped pointers fail cleanly instead of panicking.

### `sys_getpid` (`EAX = 20`)
- **Returns**: PID of the scheduler's current task.

## Trap Gate Setup (`syscall::init`)

```rust
pub fn init() {
    unsafe {
        isr_register_handler(128, syscall_dispatcher);
    }
}
```
The GDT/IDT sets vector 128 with privilege level DPL=3 (`0xEE`), allowing Ring 3 user tasks to invoke `int 0x80` without generating a General Protection Fault (`#GP`).

## Example Usage (Assembly Call)

```rust
let test_msg = "POSIX syscall test verified.\n";
unsafe {
    core::arch::asm!(
        "int 0x80",
        in("eax") 4u32,                      // SYS_WRITE
        in("ebx") 1u32,                      // stdout
        in("ecx") test_msg.as_ptr() as u32,  // Buffer pointer
        in("edx") test_msg.len() as u32,     // String length
    );
}
```

The shell command `syscall` invokes `int 0x80` from kernel context to exercise `SYS_WRITE` and `SYS_GETPID`; it does not test a Ring 3 transition. The Ring 3 demo's syscall check is described in [Processes, Scheduling, and Ring 3](processes.md).
