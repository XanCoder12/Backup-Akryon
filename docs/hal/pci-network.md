# PCI Bus & Realtek RTL8139 Network Interface

Nyxara OS includes full PCI bus enumeration and a dedicated driver for the Realtek RTL8139 Fast Ethernet controller (10/100 Mbps), enabling bare-metal and virtualized networking.

## PCI Configuration Mechanism (`hal/pci.c`)

PCI devices are queried through the x86 Configuration Mechanism #1 using two 32-bit I/O ports:
- **`0x0CF8` (`PCI_CONFIG_ADDRESS`)**: Specifies Bus, Device (Slot), Function, and Register Offset.
- **`0x0CFC` (`PCI_CONFIG_DATA`)**: Reads or writes the selected 32-bit register.

### Address Encoding
```
Bit 31:     Enable Configuration Space Access (must be 1)
Bits 30-24: Reserved (0)
Bits 23-16: PCI Bus Number (0..255)
Bits 15-11: Device / Slot Number (0..31)
Bits 10-8:  Function Number (0..7)
Bits 7-2:   Register Offset (aligned to 4 bytes)
Bits 1-0:   Always 00
```

### Bus Enumeration & Mastering
- `pci_scan_bus()`: Iterates across 256 buses, 32 device slots, and 8 functions. Valid devices return a Vendor ID other than `0xFFFF` and `0x0000`.
- `pci_enable_bus_master()`: Sets bit 0 (I/O Space Access) and bit 2 (Bus Mastering / DMA) in the PCI Command Register (`Offset 0x04`). This is required for the RTL8139 to write incoming Ethernet packets directly into host DRAM.

## Realtek RTL8139 NIC Driver (`hal/rtl8139.c`)

The RTL8139 is matched during PCI probing by Vendor ID `0x10EC` and Device ID `0x8139`.

### Register Offsets
Accessed relative to the I/O base address extracted from PCI Base Address Register 0 (BAR0):

| Offset | Name | Function |
|---|---|---|
| `0x00 - 0x05` | `MAC0 - MAC5` | Station Physical MAC Address (6 bytes) |
| `0x10 - 0x1C` | `TSD0 - TSD3` | Transmit Status / Size Descriptors (4 channels) |
| `0x20 - 0x2C` | `TSAD0 - TSAD3`| Transmit Start Address Descriptors (4 channels) |
| `0x30` | `RBSTART` | Receive Buffer Start Physical Address |
| `0x37` | `CR / COMMAND` | Command Register (Reset, Receiver Enable, Transmitter Enable) |
| `0x38` | `CAPR` | Current Address of Packet Read (Ring buffer read pointer) |
| `0x3A` | `CBR` | Current Buffer Address (Hardware write pointer) |
| `0x3C` | `IMR` | Interrupt Mask Register (`0x0005`: ROK + TOK) |
| `0x3E` | `ISR` | Interrupt Status Register (Read to detect, write 1s to clear) |
| `0x40` | `TCR` | Transmit Configuration Register |
| `0x44` | `RCR` | Receive Configuration Register |
| `0x52` | `CONFIG1` | Power Management (Writing 0x00 wakes up chip) |

### Initialization Sequence
1. **Power Up**: Write `0x00` to `CONFIG1` to disable power-saving and bring the PHY out of sleep mode.
2. **Software Reset**: Write `RTL_CMD_RESET` (`0x10`) to `COMMAND` and wait until bit 4 clears.
3. **Buffer Allocation**:
   - Receive ring buffer: 8 KB + 16 bytes + 2048 bytes wrap padding (`8192 + 16 + 2048`), 4-byte aligned.
   - Assign address to `RBSTART` (`0x30`).
   - Assign four 2 KB static buffers to `TSAD0` through `TSAD3`.
4. **Interrupt Mask**: Set `IMR` to `RTL_INT_ROK | RTL_INT_TOK` (Receive OK and Transmit OK).
5. **Receive Filter (`RCR`)**: Accept Broadcast (`0x08`), Multicast (`0x04`), and Physical Destination Match (`0x02`), with wrap mode enabled (`0x80`).
6. **Enable Engines**: Write `RTL_CMD_TE | RTL_CMD_RE` (`0x0C`) to `COMMAND`.
7. **MAC Reading**: Read 6 bytes from ports `MAC0` through `MAC5`.

### Transmission Flow
1. Copy packet bytes into current transmit buffer `tx_buffers[tx_cur]`.
2. Write packet length to `TSD0 + (tx_cur * 4)`.
3. Hardware immediately initiates DMA transmission.
4. Advance round-robin descriptor index: `tx_cur = (tx_cur + 1) % 4`.

### Reception Flow
1. Hardware issues IRQ line interrupt (`RTL_INT_ROK`).
2. Driver reads the 4-byte packet header:
   - Bits 0..15: Packet Status (ROK, CRC error, etc.).
   - Bits 16..31: Packet Length (including 4-byte CRC).
3. Copies packet data from `rx_buffer` into the upper network stack buffer.
4. Updates read pointer `CAPR = (rx_offset - 16) & 0xFFFF`.
