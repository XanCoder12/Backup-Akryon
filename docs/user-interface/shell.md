# Interactive Shell Subsystem

The Nyxara interactive shell (`rust/src/shell.rs`) provides a feature-complete line-buffered console environment resembling modern Unix shells, featuring command history, cursor movement, word-level editing shortcuts, and color rendering.

## Architecture

The shell runs in a continuous Read-Eval-Print Loop (REPL) within `shell::run_shell()`:

1. Renders the prompt: `nyxara> ` in Light Cyan.
2. Reads scancodes and control keycodes from the PS/2 keyboard ring buffer via `keyboard_getchar()`.
3. Updates the active line buffer (`[u8; 70]`) and redraws modified characters.
4. On `Enter`, commits the line to the ring buffer of command history and hands the command string to `commands::handle_command()`.

## Line Editing Keybindings

Nyxara supports standard GNU Readline / Emacs-style terminal shortcuts:

| Shortcut / Key | Function | Description |
|---|---|---|
| **`Enter` / `Return`** | Submit | Executes the command in the active buffer |
| **`Ctrl + C`** | Cancel | Discards current line buffer and begins a new prompt |
| **`Ctrl + L`** | Clear Screen | Clears the terminal and reprints the current prompt line |
| **`Left Arrow`** | Move Left | Decrements cursor column within the buffer |
| **`Right Arrow`** | Move Right | Increments cursor column toward end of input |
| **`Ctrl + Left`** | Word Left | Jumps cursor backward to the start of the previous word |
| **`Ctrl + Right`** | Word Right | Jumps cursor forward to the start of the next word |
| **`Home` / `Ctrl + A`** | Line Start | Moves cursor to column 0 |
| **`End` / `Ctrl + E`** | Line End | Moves cursor to the end of the text |
| **`Backspace`** | Delete Back | Deletes the character before the cursor and shifts buffer left |
| **`Delete` / `Ctrl + D`**| Delete Forward| Deletes character under cursor and shifts buffer left |
| **`Ctrl + W`** | Kill Word Back | Deletes the word immediately preceding the cursor |
| **`Alt + Backspace`** | Kill Word Back | Alternate keycode for deleting previous word |
| **`Ctrl + K`** | Kill to End | Erases all characters from the cursor position to end of line |
| **`Ctrl + U`** | Kill Line | Clears the entire current line buffer |
| **`Up Arrow`** | History Previous | Recalls the previous command from the history ring buffer |
| **`Down Arrow`** | History Next | Recalls the next command or restores the draft line |

## Command History

- **Capacity**: Holds the 16 most recently executed commands in a circular static array (`CommandHistory`).
- **Duplicate Suppression**: Does not record empty lines or identical back-to-back entries.
- **Draft Preservation**: Modifying a recalled history line does not overwrite the historical record until executed.

## Terminal Output Formatting

Shell output uses the `print!`, `println!`, and `print_colored!` macros declared in `rust/src/vga.rs`:
- Standard messages: White or Light Gray on Black.
- Success notifications: Light Green `[OK]`.
- Informational notices: Light Cyan `[INFO]`.
- Warnings and Prompts: Yellow or Light Cyan.
- Error alerts: Light Red `[ERROR]`.
