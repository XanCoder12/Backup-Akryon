# Kernel Entry Point and Assembly Stubs

The 32-bit assembly glue in `boot/kernel_entry.asm` serves as the bridge between the 16-bit MBR bootloader, the C Hardware Abstraction Layer, and the x86 CPU hardware interrupt subsystem.

## The Entry Point (`_start`)

Execution arrives at `_start` at physical address `0x00010000` immediately following the MBR's far jump into Protected Mode:

```assembly
[BITS 32]
global _start

_start:
    mov ax, 0x10                    ; Kernel Data Segment (0x10)
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax
    mov ss, ax

    mov esp, 0x00140000             ; Initialize Kernel Stack Top (64 KB stack)

    ; Clear the uninitialized data (.bss) section
    mov edi, bss_start
    mov ecx, bss_end
    sub ecx, edi
    xor eax, eax
    cld
    rep stosb

    mov esp, 0x00140000             ; Re-anchor stack pointer after clearing BSS
```

### 1. FPU & SSE Enabling
To support floating-point operations, hardware acceleration, and modern compiler code-generation without causing Invalid Opcode (`#UD`) or Device Not Available (`#NM`) faults:

```assembly
    mov eax, cr0
    and eax, ~(1 << 2)              ; Clear EM (Emulation)
    or eax, (1 << 1)               ; Set MP (Monitor Coprocessor)
    mov cr0, eax

    mov eax, cr4
    or eax, (3 << 9)               ; Set OSFXSR (bit 9) and OSXMMEXCPT (bit 10)
    mov cr4, eax

    fninit                          ; Reset and initialize x87 FPU state
```

### 2. Calling C Kernel Main
```assembly
    push 0x9000                     ; Push pointer to boot_info structure
    call kmain                      ; Transfer control to kernel/kmain.c
    add esp, 4

.hang:
    cli
    hlt
    jmp .hang
```

## Assembly Helper Functions

### `load_gdt_asm`
Accepts a pointer to the GDT descriptor, issues `lgdt`, and executes a far jump to reload `CS` with segment selector `0x08`:

```assembly
load_gdt_asm:
    mov eax, [esp + 4]
    lgdt [eax]
    jmp 0x08:.reload_cs
.reload_cs:
    mov ax, 0x10
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax
    mov ss, ax
    ret
```

### `load_idt_asm`
Accepts a pointer to the IDT descriptor and issues the `lidt` instruction:
```assembly
load_idt_asm:
    mov eax, [esp + 4]
    lidt [eax]
    ret
```

### `tss_flush_asm`
Loads the Task Register (TR) with selector `0x28` (TSS descriptor in GDT):
```assembly
tss_flush_asm:
    mov ax, 0x28
    ltr ax
    ret
```

## Interrupt & Exception Trampolines

x86 pushes differing amounts of data to the stack depending on whether an exception generates a hardware error code:
- **Exceptions without Error Code** (0–7, 9, 15–20, 28–31): Push a dummy error code `0` to align the stack frame.
- **Exceptions with Error Code** (8, 10–14, 21, 29, 30): The CPU automatically pushes an error code.

### Common ISR Stub (`isr_common_stub`)
```assembly
isr_common_stub:
    pusha                           ; Pushes EAX, ECX, EDX, EBX, ESP, EBP, ESI, EDI
    mov ax, ds
    push eax                        ; Save data segment

    mov ax, 0x10                    ; Load kernel data segment
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax

    push esp                        ; Pass pointer to Registers struct as argument
    call isr_handler
    add esp, 4

    pop eax                         ; Restore original data segment
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax

    popa                            ; Restore general-purpose registers
    add esp, 8                      ; Clean up error code and interrupt number
    iret                            ; Return from interrupt
```

### Hardware IRQ Trampoline (`irq_common_stub`)
For IRQ 0 through 15, the stub pushes `0` (dummy error code) and the remapped vector number (`32` through `47`), saves the CPU context, calls C `irq_handler`, and handles End-of-Interrupt (EOI) signaling to PIC 8259.

### Syscall Trampoline (`isr128`)
`isr128` hooks vector 128 (`0x80`). When a user or kernel thread executes `int 0x80`, the CPU switches to supervisor mode, constructs the `Registers` frame, and invokes the registered Rust system call dispatcher.
