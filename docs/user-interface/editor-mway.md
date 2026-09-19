# Full-Screen Text Editor (`mway`)

Nyxara OS features an integrated full-screen modal text editor called **`mway`** (`rust/src/editor.rs`). It provides interactive file editing within the console without requiring external dependencies or userland utilities.

## Launching the Editor

From the Nyxara shell, execute:
```bash
mway <filename>
```
- If the file exists in the Virtual File System (VFS), its contents are loaded into the editor buffer.
- If the file does not exist, a new empty buffer is created under that filename.

## Display Layout

The editor divides the standard 80x25 terminal screen into two distinct zones:

```
+------------------------------------------------------------------------------+
| Row 0..22: Text Editing Canvas (80 columns x 23 lines visible)               |
|            - Supports horizontal line wrapping / truncation                  |
|            - Vertical scrolling as cursor reaches screen top/bottom          |
|                                                                              |
+------------------------------------------------------------------------------+
| Row 23:    Separator Line                                                    |
+------------------------------------------------------------------------------+
| Row 24:    Status Bar (White text on Blue background)                        |
|            mway: [filename] [*] | Ln 1, Col 1 | ^S Save  ^Q Exit             |
+------------------------------------------------------------------------------+
```

## Buffer Architecture

The editor buffer is represented in `rust/src/editor.rs` as:

```rust
pub struct Editor {
    lines: Vec<Vec<u8>>,      // Dynamic collection of ASCII byte vectors
    cursor_row: usize,        // Active line index in buffer (0-indexed)
    cursor_col: usize,        // Active character offset within current line
    scroll_row: usize,        // Index of the top-most visible line
    filename: String,         // Target VFS file path
    modified: bool,           // Tracks unsaved buffer modifications
}
```

## Keyboard Shortcuts & Navigation

| Key / Combination | Action | Description |
|---|---|---|
| **`Up / Down Arrow`** | Navigate Lines | Moves cursor vertically, automatically clamping column position |
| **`Left / Right Arrow`** | Navigate Chars | Moves cursor horizontally within the line |
| **`Home` / `Ctrl + A`** | Line Start | Moves cursor to the first character of the current line |
| **`End` / `Ctrl + E`** | Line End | Moves cursor to the end of the current line |
| **`Page Up`** | Page Up | Scrolls editor view up by 20 lines |
| **`Page Down`** | Page Down | Scrolls editor view down by 20 lines |
| **`Enter` / `Return`** | Split Line | Inserts a newline, splitting current line at cursor into two |
| **`Backspace`** | Delete Backward | Deletes character before cursor; joins lines if at column 0 |
| **`Delete`** | Delete Forward | Deletes character under cursor; joins next line if at end |
| **`Tab`** | Insert Tab | Inserts 4 spaces into the current line |
| **`Ctrl + K`** | Kill to End | Erases text from cursor to the end of the current line |
| **`Ctrl + S`** | **Save File** | Flushes active buffer to VFS (`vfs::write_file`) and clears modified flag |
| **`Ctrl + Q`** | **Exit Editor** | Returns to the Nyxara shell prompt |

## VFS Persistence

When `Ctrl+S` is pressed:
1. The editor concatenates all lines in `self.lines`, delimiting them with standard Unix newline bytes (`\n`).
2. Calls `crate::vfs::write_file(&self.filename, &content)`.
3. Sets `self.modified = false` and updates the status bar with `[Saved]`.
