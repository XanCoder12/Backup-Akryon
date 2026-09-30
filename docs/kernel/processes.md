# Processes, Scheduling, and Ring 3

Nyxara has a round-robin scheduler running user tasks as isolated Ring 3 processes: each task owns a page directory and a kernel stack, and the Unix process primitives `fork`, `execve`, `exit`, and `waitpid` are wired through `int 0x80`. The userland side of this is described in [Userland](userland.md); this page covers the scheduler and privilege machinery.

## Scheduler

The scheduler is implemented in `rust/src/process.rs`. `process::init()` creates the shell as task 0. The task table holds up to eight tasks; each task record carries the pid, parent pid, state (`Ready`, `Running`, `Blocked`, `Zombie`), the saved interrupt frame pointer, its kernel stack top, and the physical address of its page directory (`0` for kernel tasks on the boot stack).

The PIT runs at 100 Hz. On each timer interrupt, `nyxara_scheduler_tick()` updates task ticks and wakes expired sleepers. It switches to the next ready task after a five-tick quantum (about 50 ms). The switch path:

1. `vmm::switch_directory(pd)` loads the next task's CR3 (skipped when unchanged).
2. `tss_set_stack(0x10, stack_top)` points `TSS.esp0` at the next task's kernel stack — kernel tasks get the boot stack top `0x00140000`.
3. `irq_set_switch_frame(frame, stack_top)` hands the frame to the IRQ stub in `boot/kernel_entry.asm`, which restores the general registers and `iret`s through a return frame rebuilt at the task's kernel stack top (or straight through the saved frame for boot-stack tasks).

`sleep(ticks)` blocks a task until a later tick; `waitpid` blocks with `wake_at = u64::MAX` so only the exiting child can wake the waiter.

## Kernel Stacks & TSS

Each task owns a 4 KB kernel stack (`STACKS`). Because `TSS.esp0` is updated on every context switch, an interrupt or `int 0x80` from Ring 3 always pushes its frame onto the owning task's kernel stack; saved frames are never clobbered by other tasks. The TSS `ss0` is `0x10`. Kernel tasks preempted in Ring 0 save their frame on the boot stack.

## Privilege Transition

The GDT provides kernel selectors `0x08`/`0x10`, user selectors `0x1B`/`0x23` (the user descriptors at indexes 3 and 4 with RPL 3), and a 32-bit TSS at selector `0x28`. Entering Ring 3 works by loading a task frame with `CS = 0x1B`, `SS = 0x23`, a user `EIP`/`ESP`, and `iret`ing during a context switch.

Two paths resume tasks from Ring 0:

- The **timer path** (`irq_common_stub` + `irq_set_switch_frame`) for preemption, as above.
- **`task_resume_asm(frame, stack_top, cr3)`** for syscall contexts that must never return to the caller — currently process exit. It loads CR3, restores the saved registers, and `iret`s either through a rebuilt frame at the task's kernel stack top or through the saved frame itself when `stack_top == 0`. The final stack pointer is kept in the `resume_esp` variable because `popa` clobbers every general register.

## Task Life Cycle

- `spawn_elf(image, name)` registers a fresh Ring 3 task with its own address space (see [Userland](userland.md)).
- `fork_current()` clones the caller: eager copy of user pages, duplicated frame with `EAX = 0` in the child.
- `exec_current()` replaces the caller's address space and rewrites the in-flight syscall frame so the return lands on the new entry point.
- `exit_current()` frees the address space, wakes a waiting parent (reaping immediately in that case, otherwise leaving a `Zombie` for `waitpid`), re-parents or reaps children, and resumes the next task via `task_resume_asm`.
- `wait_kernel(pid)` is the kernel-side polling reap used by the shell's `run` command.

`ps` prints the table including the `zombie` state and parent pid column.

## Current Limits

- `fork` copies user memory eagerly; copy-on-write is not implemented.
- The quantum and task table are fixed (8 tasks, 4 KB kernel stacks).
- `sys_read` is a stub, so user tasks cannot do blocking input yet.
- Ring 0 tasks cannot `exit` via syscall; the shell is a permanent root task.

See [System Calls](syscalls.md), [Userland](userland.md), and [Virtual Memory](vmm.md) for the related interfaces.
