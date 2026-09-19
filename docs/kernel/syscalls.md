# Unix System Calls (`int 0x80`)

Nyxara OS implements a Unix/POSIX-compatible system call interface via software interrupt vector 128 (`0x80`). This interface provides the fundamental boundary for ring transitions between user processes and kernel supervisor routines.

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

## Supported System Calls

```rust
pub const SYS_EXIT:   u32 = 1;
pub const SYS_FORK:   u32 = 2;
pub const SYS_READ:   u32 = 3;
pub const SYS_WRITE:  u32 = 4;
pub const SYS_OPEN:   u32 = 5;
pub const SYS_CLOSE:  u32 = 6;
pub const SYS_GETPID: u32 = 20;
```

### System Call Descriptions

### 1. `sys_exit` (`EAX = 1`)
- **Arguments**: `EBX`: Exit status code (`int status`).
- **Action**: Logs process termination and releases task resources.

### 2. `sys_read` (`EAX = 3`)
- **Arguments**: `EBX`: File descriptor (`int fd`), `ECX`: Destination buffer pointer (`void* buf`), `EDX`: Maximum bytes to read (`size_t count`).
- **Returns**: Number of bytes read, or negative errno on error.

### 3. `sys_write` (`EAX = 4`)
- **Arguments**: `EBX`: File descriptor (`int fd`), `ECX`: Source buffer pointer (`const void* buf`), `EDX`: Byte count (`size_t count`).
- **Behavior**:
  - `fd == 1` (stdout) or `fd == 2` (stderr): Writes characters directly to the VGA display and serial debug log.
  - Invalid descriptors return `-EBADF` (`-9`).

### 4. `sys_getpid` (`EAX = 20`)
- **Returns**: Current Process ID (returns `1` for the root kernel shell task).

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

The shell command `syscall` triggers this routine to verify kernel trapping and return behavior.
