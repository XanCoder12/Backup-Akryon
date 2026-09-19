# Master Boot Record (MBR) Bootloader

The Nyxara MBR bootloader resides in `boot/boot.asm`. It is assembled by NASM into a 512-byte raw binary that occupies the very first sector (LBA 0) of the bootable media, ending with the standard boot signature `0xAA55`.

## Key Responsibilities

1. Initialize 16-bit Real Mode segment registers and temporary stack pointer.
2. Read the multi-sector kernel image from disk into memory at physical `0x00010000`.
3. Query VESA BIOS Extensions (VBE) and configure a high-resolution linear framebuffer mode.
4. Enable the A20 address line.
5. Setup a flat Global Descriptor Table (GDT).
6. Enable CPU Protected Mode and transfer control to 32-bit kernel code.

## 16-bit Disk Loading: LBA Chunk Loader

BIOS `int 0x13, AH=0x02` (CHS read) has cylinder/head limits and cannot easily cross track boundaries or read large kernels. Nyxara uses **BIOS Extended Read (`int 0x13, AH=0x42`)** with an LBA Disk Address Packet (DAP).

```assembly
align 4
disk_address_packet:
    db 0x10                         ; Size of DAP packet (16 bytes)
    db 0x00                         ; Reserved (must be 0)
dap_num_sectors:
    dw 64                           ; Number of sectors per chunk (32 KB)
dap_offset:
    dw 0x0000                       ; Memory buffer offset (always 0x0000)
dap_segment:
    dw 0x1000                       ; Segment address (starts at 0x1000 -> phys 0x10000)
dap_lba_low:
    dd 1                            ; Starting LBA sector (1 = sector immediately after MBR)
dap_lba_high:
    dd 0                            ; Upper 32-bits of LBA
```

### Chunk Loop Logic
- **Total Sectors**: Reads 900 sectors (~450 KB) accommodating kernel code, drivers, and static data.
- **Chunk Size**: 64 sectors per call (32 KB).
- **Segment Increment**: On each successful read, `dap_segment` is advanced by `num_sectors * 32` (`shl ax, 5`), avoiding offset overflow beyond 64 KB boundaries:
  ```assembly
  add [dap_lba_low], eax
  shl ax, 5
  add [dap_segment], ax
  ```

## VESA BIOS Extensions (VBE) Video Initialization

Before transitioning to protected mode (where 16-bit BIOS interrupts are no longer accessible), the MBR configures the graphics mode:

1. Mode Detection:
   - Tries mode `0x143` (800x600, 32-bit truecolor).
   - Fallback to mode `0x144` (1024x768, 32-bit truecolor).
   - Fallback to mode `0x118` (1024x768, 24/32-bit).
2. Mode Query: Uses `int 0x10, AX=0x4F01` into a 256-byte scratch buffer at `0x7E00`.
3. Linear Framebuffer (LFB) Activation: Sets bit 14 (`BX = mode | 0x4000`) and calls `int 0x10, AX=0x4F02`.
4. Boot Information Block (`0x9000`): Saves framebuffer metadata for the 32-bit kernel:
   - `0x9000`: Magic number `0x414B5259` ("AKRY").
   - `0x9004`: `PhysBasePtr` (32-bit physical address of video RAM).
   - `0x9008`: `XResolution` (uint16_t).
   - `0x900A`: `YResolution` (uint16_t).
   - `0x900C`: `BytesPerScanLine` (uint16_t).
   - `0x900E`: `BitsPerPixel` (uint8_t).
   - `0x900F`: `is_active` flag (1 = active, 0 = fallback to VGA text mode).

## Fast A20 Gate Activation

To prevent address line 20 from being masked (which causes memory wrapping at 1 MB):

```assembly
enable_a20:
    in al, 0x92
    test al, 2
    jnz .done
    or al, 2
    and al, 0xFE
    out 0x92, al
.done:
    ret
```

## Protected Mode Switch

1. Disable interrupts: `cli`.
2. Load temporary MBR GDT descriptor: `lgdt [gdt_descriptor]`.
3. Set PE bit in `CR0`:
   ```assembly
   mov eax, cr0
   or eax, 1
   mov cr0, eax
   ```
4. Far jump into 32-bit code segment:
   ```assembly
   jmp 0x08:protected_mode_entry
   ```
5. Initialize 32-bit segments (`DS=0x10`, `SS=0x10`, `ESP=0x90000`), then jump directly to the kernel entry address:
   ```assembly
   jmp 0x10000
   ```
