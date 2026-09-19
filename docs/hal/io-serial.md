# Port I/O and Serial Debug Logger

Direct communication with motherboard chipset registers and legacy hardware in x86 architectures is performed through dedicated I/O ports. Nyxara encapsulates these interactions in `hal/io.c` and builds a robust diagnostic serial console in `hal/serial.c`.

## Port I/O Operations (`hal/io.c`)

Because port I/O requires privileged CPU instructions (`in`, `out`), the HAL provides inline assembly wrappers:

```c
// 8-bit Byte Transfer
void outb(uint16_t port, uint8_t val) {
    __asm__ volatile ("outb %0, %1" : : "a"(val), "Nd"(port));
}
uint8_t inb(uint16_t port) {
    uint8_t ret;
    __asm__ volatile ("inb %1, %0" : "=a"(ret) : "Nd"(port));
    return ret;
}

// 16-bit Word Transfer
void outw(uint16_t port, uint16_t val) {
    __asm__ volatile ("outw %0, %1" : : "a"(val), "Nd"(port));
}
uint16_t inw(uint16_t port) {
    uint16_t ret;
    __asm__ volatile ("inw %1, %0" : "=a"(ret) : "Nd"(port));
    return ret;
}

// 32-bit Dword Transfer
void outl(uint16_t port, uint32_t val) {
    __asm__ volatile ("outl %0, %1" : : "a"(val), "Nd"(port));
}
uint32_t inl(uint16_t port) {
    uint32_t ret;
    __asm__ volatile ("inl %1, %0" : "=a"(ret) : "Nd"(port));
    return ret;
}
```

### CPU Control Primitives
- `cli()`: Executes `cli` instruction, disabling maskable hardware interrupts.
- `sti()`: Executes `sti` instruction, enabling maskable hardware interrupts.
- `hlt()`: Executes `hlt` instruction, pausing CPU execution until the next interrupt arrives.
- `io_wait()`: Performs an arbitrary write to unused port `0x80` to introduce a brief cycle delay (~1-4 microseconds) for slow legacy bus synchronization (e.g. during PIC remapping).

## UART 16550 Serial Driver (`hal/serial.c`)

Nyxara OS uses COM1 (`0x3F8`) as the primary out-of-band debugging and logging channel. Serial output is streamed to QEMU stdio or saved to `serial.log`.

### Port Offsets for COM1 (`0x3F8`)
- `COM1 + 0` (`0x3F8`): Data Register (Transmit/Receive) or Baud Divisor Low Byte (when DLAB=1).
- `COM1 + 1` (`0x3F9`): Interrupt Enable Register or Baud Divisor High Byte.
- `COM1 + 2` (`0x3FA`): FIFO Control Register (FCR).
- `COM1 + 3` (`0x3FB`): Line Control Register (LCR). Bit 7 is DLAB (Divisor Latch Access Bit).
- `COM1 + 4` (`0x3FC`): Modem Control Register (MCR).
- `COM1 + 5` (`0x3FD`): Line Status Register (LSR). Bit 5 indicates Transmit Empty.

### Initialization Sequence (`serial_init`)
```c
int serial_init(void) {
    outb(COM1_PORT + 1, 0x00);    // Disable interrupts
    outb(COM1_PORT + 3, 0x80);    // Enable DLAB
    outb(COM1_PORT + 0, 0x03);    // Set divisor to 3 (115200 / 3 = 38400 baud)
    outb(COM1_PORT + 1, 0x00);
    outb(COM1_PORT + 3, 0x03);    // 8 bits, no parity, 1 stop bit (8N1)
    outb(COM1_PORT + 2, 0xC7);    // Enable FIFO, clear RX/TX, 14-byte threshold
    outb(COM1_PORT + 4, 0x0B);    // Enable IRQs, set RTS/DSR
    return 0;
}
```

### Output Functions
- `serial_putchar(char c)`: Waits for `LSR & 0x20` (Transmitter Holding Register Empty) before writing byte.
- `serial_puts(const char* str)`: Outputs null-terminated string, automatically converting `\n` to `\r\n`.
- `serial_puthex(uint32_t val)`: Prints 32-bit hexadecimal value with `0x` prefix.
- `serial_puthex16(uint16_t val)` & `serial_puthex8(uint8_t val)`.
- `serial_putdec(uint32_t val)`: Converts and prints unsigned decimal integers.
