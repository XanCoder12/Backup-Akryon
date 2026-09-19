# Framebuffer Engine & Graphical Applications

Nyxara OS includes a high-performance 32-bit Linear Framebuffer (LFB) graphics engine (`rust/src/framebuffer.rs`), an embedded Terminus 8x16 bitmap font (`rust/src/font.rs`), a software mouse cursor sprite compositor, and an interactive graphical drawing program (`paint`).

## Linear Framebuffer (LFB) Architecture

When booted into VBE graphics mode (800x600 or 1024x768 at 32 bpp), video memory is treated as a continuous linear array of 32-bit TrueColor pixels:

- **Color Encoding**: ARGB `0x00RRGGBB` (8 bits per channel).
- **Pixel Offset Formula**:
  ```rust
  let offset = y * bytes_per_scanline + x * (bits_per_pixel / 8);
  ```
- **Catppuccin Mocha Palette**: Nyxara maps standard 16-color VGA indices into modern Catppuccin Mocha hex codes (e.g. Lavender `0xB4BEFE`, Sapphire `0x74C7EC`, Mauve `0xCBA6F7`, Text `0xCDD6F4`, Base `0x000000`).

## Embedded 8x16 Bitmap Font Engine

Text is rendered into the graphical framebuffer via `rust/src/font.rs`:
- **Font Dimensions**: 8 pixels wide by 16 pixels high per glyph (`FONT_W = 8`, `FONT_H = 16`).
- **Glyph Bitmap**: 16 bytes per ASCII character, where each byte represents one row of 8 horizontal pixels.
- **Glyph Blitting**:
  ```rust
  for row in 0..16 {
      let byte = glyph[row];
      for col in 0..8 {
          if (byte & (0x80 >> col)) != 0 {
              put_pixel(x + col, y + row, fg_color);
          } else {
              put_pixel(x + col, y + row, bg_color);
          }
      }
  }
  ```
- Transforms high-resolution display modes into a sharp, legible terminal grid (e.g. 100 columns by 37 rows on 800x600, or 128 columns by 48 rows on 1024x768).

## Mouse Pointer Sprite Compositor

To display a graphical mouse cursor without flickering or overwriting underlying interface content:
1. **Background Preservation**: Before drawing the mouse arrow sprite, the framebuffer pixels beneath the sprite area (16x16 pixels) are copied into a temporary preservation buffer.
2. **Sprite Overlay**: The mouse arrow shape is drawn onto the screen with transparency support.
3. **Cursor Restoration**: When the mouse moves to a new coordinate, the preserved background pixels are restored to their original location, and the process repeats at the new position.

## Interactive Drawing Application (`paint`)

The `paint` command demonstrates real-time graphical input and mouse interaction:

- **Launch Command**: `paint`
- **Interaction**:
  - **Left Mouse Click + Drag**: Draws colored brush strokes directly onto the canvas.
  - **Right Mouse Click**: Erases pixels / draws background color.
  - **Number Keys (1–8)**: Selects active drawing color from the palette.
  - **C Key**: Clears the drawing canvas.
  - **ESC Key**: Closes the application and restores the shell console.
