# PS/2 Keyboard and Mouse Drivers

Nyxara OS interacts with user input hardware through the standard Intel 8042 PS/2 controller using Port `0x60` (Data) and Port `0x64` (Status/Command).

## PS/2 Keyboard Driver (`hal/keyboard.c`)

The keyboard driver handles IRQ 1 (Vector 33), decoding Scancode Set 1 make and break codes and feeding a circular ring buffer.

### Ring Buffer Architecture
```c
#define KEYBOARD_BUFFER_SIZE 256
static volatile uint16_t key_buffer[KEYBOARD_BUFFER_SIZE];
static volatile uint32_t buf_head = 0;
static volatile uint32_t buf_tail = 0;
```
- Non-blocking push and pop operations prevent dropped keypresses during CPU workloads.
- Character extraction: `keyboard_getchar()` blocks or polls until `buf_head != buf_tail`.

### Scancode Decoding & Modifiers
- **Make Codes** (`< 0x80`): Trigger key-press actions.
- **Break Codes** (`>= 0x80`): Released keys (make code + `0x80`).
- **Modifier Tracking**:
  - `Shift` (Left `0x2A`, Right `0x36`) toggles between `kbd_us_lower` and `kbd_us_upper` tables.
  - `Caps Lock` (`0x3A`) toggles capitalization for alphabetic characters.
  - `Ctrl` (Left `0x1D`) and `Alt` (Left `0x38`) generate control codes (e.g., Ctrl+S = `0x13`, Ctrl+Q = `0x11`).
- **Extended Scancodes (`0xE0` Prefix)**:
  - Cursor arrows: Up (`0xE048`), Down (`0xE050`), Left (`0xE04B`), Right (`0xE04D`).
  - Navigation: Home (`0xE047`), End (`0xE04F`), Page Up (`0xE049`), Page Down (`0xE051`), Delete (`0xE053`).

## PS/2 Mouse Driver (`hal/mouse.c`)

The mouse driver handles IRQ 12 (Vector 44), interfacing with the auxiliary PS/2 device.

### Hardware Initialization
1. Enable auxiliary mouse port: write `0xA8` to port `0x64`.
2. Enable mouse IRQ 12 in the 8042 controller configuration byte (read byte via `0x20`, bitwise OR with `0x02`, write back via `0x60`).
3. Reset to defaults: send command `0xF6` to mouse via port `0xD4`.
4. Enable packet streaming: send command `0xF4` to mouse.

### Packet Parsing (3-Byte Streaming Protocol)
Each movement or button event produces a 3-byte stream:

```
Byte 0: [ Y_Ovf | X_Ovf | Y_Sign | X_Sign | 1 | MidBtn | RightBtn | LeftBtn ]
Byte 1: [                 X Movement Delta (8-bit)                           ]
Byte 2: [                 Y Movement Delta (8-bit)                           ]
```

- **Synchronization Check**: Byte 0 must have Bit 3 set to 1 (`!(b & 0x08)`). If violated, the packet index resets to 0 to resynchronize the stream.
- **Sign Extension**:
  ```c
  int32_t dx = (int32_t)packet[1];
  int32_t dy = (int32_t)packet[2];

  if (packet[0] & 0x10) dx |= 0xFFFFFF00; // Sign-extend X
  if (packet[0] & 0x20) dy |= 0xFFFFFF00; // Sign-extend Y
  ```
- **Coordinate Tracking & Clamping**:
  - Position is updated with `mouse_x += dx` and `mouse_y -= dy` (inverting Y delta to match screen space coordinates).
  - Clamped within `[0, mouse_max_x]` and `[0, mouse_max_y]`.

### Exported C APIs
- `mouse_init()`: Configures PS/2 controller and enables IRQ 12.
- `mouse_get_x()` & `mouse_get_y()`: Return current cursor coordinates.
- `mouse_get_buttons()`: Returns bitmask (`0x01` Left, `0x02` Right, `0x04` Middle).
- `mouse_set_bounds(int32_t max_x, int32_t max_y)`: Sets screen resolution bounds.
