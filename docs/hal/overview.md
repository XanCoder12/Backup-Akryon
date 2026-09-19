# Hardware Abstraction Layer (HAL) Overview

The Hardware Abstraction Layer (HAL) provides a clean, portable interface between raw x86 hardware devices and high-level operating system subsystems. It is implemented in freestanding C (`hal/*.c`) with header declarations in `hal/*.h`, unified under `hal/hal.h`.

## Design Principles

1. **Freestanding Environment**
   - Compiled with `-ffreestanding -nostdlib -nostdinc -m32 -fno-pie -fno-stack-protector`.
   - Independent of external runtime libraries, glibc, or host system headers.
   - All standard primitive types (`uint8_t`, `uint16_t`, `uint32_t`, `uint64_t`, `size_t`, `ssize_t`) are explicitly defined in `hal/types.h`.

2. **Unified Master Header (`hal/hal.h`)**
   Includes all hardware subsystem definitions:
   ```c
   #include "io.h"
   #include "vga.h"
   #include "gdt.h"
   #include "idt.h"
   #include "isr.h"
   #include "timer.h"
   #include "keyboard.h"
   #include "serial.h"
   #include "pci.h"
   #include "rtl8139.h"
   #include "rtc.h"
   #include "fb.h"
   #include "mouse.h"
   ```

3. **Freestanding Memory and String Primitives (`hal/string.c`)**
   Provides zero-dependency implementations of essential memory functions with 32-bit word alignment optimizations:
   - `void* memset(void* dest, int val, size_t count)`: Optimized with 4-byte word writes when aligned.
   - `void* memcpy(void* dest, const void* src, size_t count)`: Dual 32-bit aligned block copies.
   - `void* memmove(void* dest, const void* src, size_t count)`: Overlap-safe memory transfer.
   - `int memcmp(const void* s1, const void* s2, size_t count)` & `int bcmp(...)`.
   - `size_t strlen(const char* str)`.
   - `int strcmp(const char* s1, const char* s2)` & `int strncmp(...)`.

## Boot Information Contract (`boot_info_t`)

The bootloader transmits detected display properties to the HAL via a packed structure allocated at physical address `0x9000`:

```c
typedef struct {
    uint32_t magic;               // Must equal 0x414B5259 ("AKRY")
    uint32_t phys_base_ptr;       // 32-bit physical address of Linear Framebuffer
    uint16_t x_resolution;        // Horizontal resolution (e.g., 800 or 1024)
    uint16_t y_resolution;        // Vertical resolution (e.g., 600 or 768)
    uint16_t bytes_per_scanline;  // Stride (e.g., 800 * 4 = 3200 bytes)
    uint8_t  bits_per_pixel;      // Color depth (e.g., 32 bpp TrueColor)
    uint8_t  is_active;           // 1 if VBE mode was successfully initialized
} __attribute__((packed)) boot_info_t;
```

## Initialization Order (`hal_init`)

The initialization order in `kernel/kmain.c` is strictly sequential to guarantee stable interrupt handling:

1. `gdt_init()`: Set up descriptor tables and Task State Segment.
2. `idt_init()`: Populate 256 interrupt gates and remap PIC 8259.
3. `fb_init(boot_info)`: Initialize graphics framebuffer, or fallback to `vga_init()`.
4. `serial_init()`: Initialize UART COM1 for diagnostic logging.
5. `timer_init(100)`: Set PIT timer to 100 Hz (10ms per tick).
6. `keyboard_init()`: Enable PS/2 keyboard IRQ 1 with ring buffer.
7. `mouse_init()`: Enable PS/2 mouse IRQ 12 streaming mode.
8. `pci_init()` & `rtl8139_init()`: Discover network card on PCI bus and configure buffers.
9. `rtc_init()`: Initialize CMOS real-time clock.
10. `sti()`: Enable hardware interrupts.
