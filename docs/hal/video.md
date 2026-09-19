# Video Drivers: VGA Text Console and VBE Framebuffer

Nyxara OS features dual-mode display support: standard 80x25 character-cell VGA text mode (`0xB8000`) and high-resolution VBE 32-bit Linear Framebuffer (LFB) graphics mode.

## VGA Text Mode Console (`hal/vga.c`)

When VBE graphics are disabled or unavailable, the system defaults to legacy VGA text mode (80 columns by 25 rows).

### Hardware Memory Buffer
- **Base Physical Address**: `0x000B8000`.
- **Dimensions**: 80 columns × 25 rows = 2,000 character cells (4,000 bytes).
- **Cell Structure**: 16-bit word containing an 8-bit ASCII character code and an 8-bit color attribute byte:
  ```
  Bit 15..12: Background Color (0..15)
  Bit 11..8:  Foreground Color (0..15)
  Bit 7..0:   ASCII Character Code
  ```

### Color Palette (16 Standard CGA/VGA Colors)
```c
typedef enum {
    VGA_COLOR_BLACK = 0,
    VGA_COLOR_BLUE = 1,
    VGA_COLOR_GREEN = 2,
    VGA_COLOR_CYAN = 3,
    VGA_COLOR_RED = 4,
    VGA_COLOR_MAGENTA = 5,
    VGA_COLOR_BROWN = 6,
    VGA_COLOR_LIGHT_GREY = 7,
    VGA_COLOR_DARK_GREY = 8,
    VGA_COLOR_LIGHT_BLUE = 9,
    VGA_COLOR_LIGHT_GREEN = 10,
    VGA_COLOR_LIGHT_CYAN = 11,
    VGA_COLOR_LIGHT_RED = 12,
    VGA_COLOR_LIGHT_MAGENTA = 13,
    VGA_COLOR_LIGHT_BROWN = 14,
    VGA_COLOR_WHITE = 15,
} vga_color_t;
```

### CRT Controller Hardware Cursor
The hardware blinking cursor is controlled via CRT Controller ports `0x3D4` (Index) and `0x3D5` (Data):
```c
void vga_update_cursor(void) {
    uint16_t pos = (uint16_t)(cursor_row * VGA_WIDTH + cursor_col);
    outb(0x3D4, 0x0F);                  // Cursor Location Low Register
    outb(0x3D5, (uint8_t)(pos & 0xFF));
    outb(0x3D4, 0x0E);                  // Cursor Location High Register
    outb(0x3D5, (uint8_t)((pos >> 8) & 0xFF));
}
```

### Scrolling Engine
When `cursor_row` exceeds row 24, `vga_scroll()` shifts rows 1..24 up by 80 cells using memory copying and clears row 24 with space characters.

## VBE Linear Framebuffer (LFB) Interface (`hal/fb.c`)

When VBE mode `0x143` (800x600) or `0x144` (1024x768) is negotiated during boot:
1. `fb_init(boot_info)` validates the magic signature `0x414B5259` and `is_graphical` flag.
2. The physical framebuffer address (`phys_base_ptr`) is stored.
3. Higher layers in Rust (`rust/src/framebuffer.rs`) map and render direct TrueColor 32-bpp pixels (ARGB `0x00RRGGBB`).
4. If initialization fails, `fb_active` remains 0, prompting automatic fallback to `vga_init()`.
