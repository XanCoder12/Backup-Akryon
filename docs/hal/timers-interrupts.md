# Timers, Scheduling Ticks, and Real-Time Clock

Accurate timekeeping in Nyxara OS relies on two distinct hardware subsystems: the Programmable Interval Timer (PIT 8254) for high-precision CPU ticks and sleep delays, and the Real-Time Clock (RTC/CMOS) for persistent calendar date and wall-clock time.

## Programmable Interval Timer (PIT 8254)

The PIT driver (`hal/timer.c`) drives IRQ 0 (Vector 32) at 100 Hz (1 tick per 10 milliseconds).

### Hardware Architecture
- **Base Oscillator Frequency**: `1,193,182 Hz`.
- **Operating Channel**: Channel 0 (connected to PIC IRQ 0).
- **Mode Control Port**: `0x43`.
- **Channel 0 Data Port**: `0x40`.

### Configuration (`timer_init`)
```c
void timer_init(uint32_t freq) {
    if (freq == 0) freq = 100;
    isr_register_handler(32, timer_callback);

    uint32_t divisor = 1193182 / freq;

    // Command 0x36: Channel 0, Access Mode Lobeyte/Hibyte, Mode 3 (Square Wave)
    outb(0x43, 0x36);
    outb(0x40, (uint8_t)(divisor & 0xFF));
    outb(0x40, (uint8_t)((divisor >> 8) & 0xFF));
}
```

### Time Tracking APIs
- `timer_get_ticks()`: Returns raw 32-bit tick counter incremented on every IRQ 0.
- `timer_get_uptime_seconds()`: Returns integer uptime in seconds (`ticks / 100`).
- `timer_get_uptime_ms()`: Returns uptime in milliseconds (`(ticks * 1000) / 100`).
- `timer_sleep_ms(uint32_t ms)`: Halts the CPU using `hlt` instructions inside a tick evaluation loop until the specified duration expires, preventing high CPU utilization during sleep.

## Real-Time Clock (RTC / CMOS)

The RTC driver (`hal/rtc.c`) reads battery-backed CMOS registers to provide date and wall-clock time (`date` and `time` shell commands).

### Hardware Register Layout
Accessed via Index Address Port `0x70` and Data Port `0x71`:

| Register Index | Function | Description |
|---|---|---|
| `0x00` | Seconds | Range 00–59 |
| `0x02` | Minutes | Range 00–59 |
| `0x04` | Hours | Range 00–23 (or 01–12 with AM/PM flag) |
| `0x07` | Day of Month | Range 01–31 |
| `0x08` | Month | Range 01–12 |
| `0x09` | Year | Last two digits (00–99) |
| `0x32` | Century | Century digits (e.g., 20) |
| `0x0A` | Status Register A | Bit 7 is Update-In-Progress (UIP) |
| `0x0B` | Status Register B | Bit 1 (24-hour mode), Bit 2 (Binary vs BCD format) |

### Non-Maskable Interrupt (NMI) Protection
Bit 7 of port `0x70` controls NMI disable:
```c
static uint8_t cmos_read(uint8_t reg) {
    outb(CMOS_ADDRESS_PORT, (uint8_t)(0x80 | reg)); // Set bit 7 to disable NMI
    return inb(CMOS_DATA_PORT);
}
```

### BCD Decoding and Synchronization
1. **Wait for Ready**: Reads Status Register A to ensure Bit 7 (`UIP`) is zero before sampling values, avoiding reading mid-tick register state.
2. **Binary Coded Decimal Conversion**: If Bit 2 of Status Register B indicates BCD encoding, values are decoded:
   ```c
   uint8_t bcd_to_bin(uint8_t bcd) {
       return (uint8_t)(((bcd >> 4) * 10) + (bcd & 0x0F));
   }
   ```
3. **Century Calculation**: Combines century register (`0x32`) with year (`0x09`). If century is unavailable or 0, defaults to `2000 + year`.
