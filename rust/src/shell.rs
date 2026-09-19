use crate::vga::{self, Color};
use crate::commands;
use crate::{print_colored, println};

extern "C" {
    fn keyboard_getchar() -> u16;
    fn keyboard_has_char() -> bool;
}

// Special extended keycodes matching hal/keyboard.h
pub const KEY_UP: u16 = 0x0100;
pub const KEY_DOWN: u16 = 0x0101;
pub const KEY_LEFT: u16 = 0x0102;
pub const KEY_RIGHT: u16 = 0x0103;
pub const KEY_HOME: u16 = 0x0104;
pub const KEY_END: u16 = 0x0105;
pub const KEY_PAGE_UP: u16 = 0x0106;
pub const KEY_PAGE_DOWN: u16 = 0x0107;
pub const KEY_INSERT: u16 = 0x0108;
pub const KEY_DELETE: u16 = 0x0109;
pub const KEY_CTRL_LEFT: u16 = 0x010A;
pub const KEY_CTRL_RIGHT: u16 = 0x010B;
pub const KEY_ALT_DELETE: u16 = 0x010C;
pub const KEY_ALT_BACKSPACE: u16 = 0x010D;
pub const KEY_SHIFT_LEFT: u16 = 0x010E;
pub const KEY_SHIFT_RIGHT: u16 = 0x010F;
pub const KEY_SHIFT_HOME: u16 = 0x0110;
pub const KEY_SHIFT_END: u16 = 0x0111;
pub const KEY_SHIFT_TAB: u16 = 0x0112;

// Control character constants
pub const KEY_CTRL_A: u16 = 0x0001; // Beginning of line (Home)
pub const KEY_CTRL_C: u16 = 0x0003; // Cancel / Interrupt (^C)
pub const KEY_CTRL_D: u16 = 0x0004; // Delete char at cursor
pub const KEY_CTRL_E: u16 = 0x0005; // End of line (End)
pub const KEY_BACKSPACE: u16 = 0x0008; // Backspace (\b)
pub const KEY_TAB: u16 = 0x0009; // Tab (\t)
pub const KEY_ENTER: u16 = 0x000A; // Enter (\n)
pub const KEY_CTRL_K: u16 = 0x000B; // Kill from cursor to end of line
pub const KEY_CTRL_L: u16 = 0x000C; // Clear screen (^L)
pub const KEY_RETURN: u16 = 0x000D; // Carriage Return (\r)
pub const KEY_CTRL_U: u16 = 0x0015; // Kill entire line
pub const KEY_CTRL_W: u16 = 0x0017; // Delete previous word
pub const KEY_CTRL_S: u16 = 0x0013; // Save (^S)
pub const KEY_CTRL_Q: u16 = 0x0011; // Quit (^Q)
pub const KEY_DEL_CHAR: u16 = 0x007F; // ASCII Del


const HISTORY_CAPACITY: usize = 16;
const MAX_CMD_LEN: usize = 70;

struct CommandHistory {
    entries: [[u8; MAX_CMD_LEN]; HISTORY_CAPACITY],
    lens: [usize; HISTORY_CAPACITY],
    count: usize,
}

impl CommandHistory {
    const fn new() -> Self {
        Self {
            entries: [[0; MAX_CMD_LEN]; HISTORY_CAPACITY],
            lens: [0; HISTORY_CAPACITY],
            count: 0,
        }
    }

    fn push(&mut self, cmd: &[u8]) {
        if cmd.is_empty() {
            return;
        }
        // Avoid duplicate consecutive history entries
        if self.count > 0 {
            let last_len = self.lens[self.count - 1];
            if last_len == cmd.len() && &self.entries[self.count - 1][..last_len] == cmd {
                return;
            }
        }

        if self.count < HISTORY_CAPACITY {
            let idx = self.count;
            self.entries[idx][..cmd.len()].copy_from_slice(cmd);
            self.lens[idx] = cmd.len();
            self.count += 1;
        } else {
            // Shift history entries when buffer is full
            for i in 0..HISTORY_CAPACITY - 1 {
                self.entries[i] = self.entries[i + 1];
                self.lens[i] = self.lens[i + 1];
            }
            let idx = HISTORY_CAPACITY - 1;
            self.entries[idx][..cmd.len()].copy_from_slice(cmd);
            self.lens[idx] = cmd.len();
        }
    }

    fn get(&self, idx: usize) -> Option<&[u8]> {
        if idx < self.count {
            Some(&self.entries[idx][..self.lens[idx]])
        } else {
            None
        }
    }
}

pub fn run_shell() -> ! {
    let mut history = CommandHistory::new();
    let mut buffer = [0u8; MAX_CMD_LEN];
    let mut len: usize = 0;
    let mut cursor: usize = 0;
    let mut selection_anchor: Option<usize> = None;

    let mut draft_buffer = [0u8; MAX_CMD_LEN];
    let mut draft_len: usize = 0;
    let mut hist_index: Option<usize> = None;

    print_prompt();
    let (mut prompt_x, mut prompt_y) = vga::get_cursor();

    loop {
        while unsafe { !keyboard_has_char() } {
            crate::net::poll();
            unsafe {
                core::arch::asm!("hlt");
            }
        }

        let key = unsafe { keyboard_getchar() };

        match key {
            // Enter key: execute command
            KEY_ENTER | KEY_RETURN => {
                set_input_cursor(prompt_x, prompt_y, len);
                println!();

                if len > 0 {
                    let cmd_slice = &buffer[..len];
                    history.push(cmd_slice);
                    if let Ok(cmd_str) = core::str::from_utf8(cmd_slice) {
                        commands::handle_command(cmd_str);
                    }
                    len = 0;
                    cursor = 0;
                }

                hist_index = None;
                print_prompt();
                let (nx, ny) = vga::get_cursor();
                prompt_x = nx;
                prompt_y = ny;
            }


            // Ctrl + C: Cancel current line & start new prompt
            KEY_CTRL_C => {
                set_input_cursor(prompt_x, prompt_y, len);
                print_colored!(Color::LightRed, Color::Black, "^C\n");
                len = 0;
                cursor = 0;
                hist_index = None;
                print_prompt();
                let (nx, ny) = vga::get_cursor();
                prompt_x = nx;
                prompt_y = ny;
            }

            // Ctrl + L: Clear screen and restore prompt & current line buffer
            KEY_CTRL_L => {
                vga::clear_screen();
                print_prompt();
                let (nx, ny) = vga::get_cursor();
                prompt_x = nx;
                prompt_y = ny;
                redraw_line(prompt_x, prompt_y, &buffer, len, cursor, 0, None);
            }

            // Left Arrow: Move cursor left
            KEY_LEFT => {
                cursor = cursor.saturating_sub(1);
                selection_anchor = None;
                redraw_line(prompt_x, prompt_y, &buffer, len, cursor, len, None);
            }

            // Right Arrow: Move cursor right
            KEY_RIGHT => {
                cursor = (cursor + 1).min(len);
                selection_anchor = None;
                redraw_line(prompt_x, prompt_y, &buffer, len, cursor, len, None);
            }

            // Shift navigation: extend the current selection
            KEY_SHIFT_LEFT => {
                if selection_anchor.is_none() { selection_anchor = Some(cursor); }
                cursor = cursor.saturating_sub(1);
                redraw_line(prompt_x, prompt_y, &buffer, len, cursor, len, selection_anchor);
            }
            KEY_SHIFT_RIGHT => {
                if selection_anchor.is_none() { selection_anchor = Some(cursor); }
                cursor = (cursor + 1).min(len);
                redraw_line(prompt_x, prompt_y, &buffer, len, cursor, len, selection_anchor);
            }
            KEY_SHIFT_HOME => {
                if selection_anchor.is_none() { selection_anchor = Some(cursor); }
                cursor = 0;
                redraw_line(prompt_x, prompt_y, &buffer, len, cursor, len, selection_anchor);
            }
            KEY_SHIFT_END => {
                if selection_anchor.is_none() { selection_anchor = Some(cursor); }
                cursor = len;
                redraw_line(prompt_x, prompt_y, &buffer, len, cursor, len, selection_anchor);
            }

            // Ctrl+Arrow: Move by one word
            KEY_CTRL_LEFT => {
                while cursor > 0 && buffer[cursor - 1] == b' ' {
                    cursor -= 1;
                }
                while cursor > 0 && buffer[cursor - 1] != b' ' {
                    cursor -= 1;
                }
                selection_anchor = None;
                redraw_line(prompt_x, prompt_y, &buffer, len, cursor, len, None);
            }
            KEY_CTRL_RIGHT => {
                while cursor < len && buffer[cursor] != b' ' {
                    cursor += 1;
                }
                while cursor < len && buffer[cursor] == b' ' {
                    cursor += 1;
                }
                selection_anchor = None;
                redraw_line(prompt_x, prompt_y, &buffer, len, cursor, len, None);
            }

            // Home / Ctrl+A: Jump to beginning of line
            KEY_HOME | KEY_CTRL_A => {
                cursor = 0;
                selection_anchor = None;
                redraw_line(prompt_x, prompt_y, &buffer, len, cursor, len, None);
            }

            // End / Ctrl+E: Jump to end of line
            KEY_END | KEY_CTRL_E => {
                cursor = len;
                selection_anchor = None;
                redraw_line(prompt_x, prompt_y, &buffer, len, cursor, len, None);
            }

            // Up Arrow: Navigate command history (previous)
            KEY_UP => {
                if history.count > 0 {
                    let next_idx = match hist_index {
                        None => {
                            // Save current draft
                            draft_buffer[..len].copy_from_slice(&buffer[..len]);
                            draft_len = len;
                            history.count - 1
                        }
                        Some(i) if i > 0 => i - 1,
                        Some(i) => i,
                    };

                    hist_index = Some(next_idx);
                    if let Some(entry) = history.get(next_idx) {
                        let old_len = len;
                        buffer[..entry.len()].copy_from_slice(entry);
                        len = entry.len();
                        cursor = len;
                        redraw_line(prompt_x, prompt_y, &buffer, len, cursor, old_len, None);
                    }
                }
            }

            // Down Arrow: Navigate command history (next)
            KEY_DOWN => {
                if let Some(idx) = hist_index {
                    if idx + 1 < history.count {
                        let next_idx = idx + 1;
                        hist_index = Some(next_idx);
                        if let Some(entry) = history.get(next_idx) {
                            let old_len = len;
                            buffer[..entry.len()].copy_from_slice(entry);
                            len = entry.len();
                            cursor = len;
                            redraw_line(prompt_x, prompt_y, &buffer, len, cursor, old_len, None);
                        }
                    } else {
                        // Restore draft buffer
                        hist_index = None;
                        let old_len = len;
                        buffer[..draft_len].copy_from_slice(&draft_buffer[..draft_len]);
                        len = draft_len;
                        cursor = len;
                        redraw_line(prompt_x, prompt_y, &buffer, len, cursor, old_len, None);
                    }
                }
            }

            // Backspace: Delete character before cursor or selection
            KEY_BACKSPACE | KEY_DEL_CHAR => {
                let old_len = len;
                if delete_selection(&mut buffer, &mut len, &mut cursor, &mut selection_anchor) {
                    redraw_line(prompt_x, prompt_y, &buffer, len, cursor, old_len, None);
                } else if cursor > 0 {
                    let old_len = len;
                    for i in cursor..len {
                        buffer[i - 1] = buffer[i];
                    }
                    cursor -= 1;
                    len -= 1;
                    selection_anchor = None;
                    redraw_line(prompt_x, prompt_y, &buffer, len, cursor, old_len, None);
                }
            }

            // Delete / Ctrl+D: Delete character at cursor
            KEY_DELETE | KEY_CTRL_D => {
                let old_len = len;
                if delete_selection(&mut buffer, &mut len, &mut cursor, &mut selection_anchor) {
                    redraw_line(prompt_x, prompt_y, &buffer, len, cursor, old_len, None);
                } else if cursor < len {
                    let old_len = len;
                    for i in (cursor + 1)..len {
                        buffer[i - 1] = buffer[i];
                    }
                    len -= 1;
                    selection_anchor = None;
                    redraw_line(prompt_x, prompt_y, &buffer, len, cursor, old_len, None);
                }
            }

            // Alt+Backspace: Delete the previous word
            KEY_ALT_BACKSPACE => {
                if cursor > 0 {
                    let old_len = len;
                    let end = cursor;
                    while cursor > 0 && buffer[cursor - 1] == b' ' {
                        cursor -= 1;
                    }
                    while cursor > 0 && buffer[cursor - 1] != b' ' {
                        cursor -= 1;
                    }
                    let deleted_count = end - cursor;
                    for i in end..len {
                        buffer[i - deleted_count] = buffer[i];
                    }
                    len -= deleted_count;
                    redraw_line(prompt_x, prompt_y, &buffer, len, cursor, old_len, None);
                }
            }

            // Alt+Delete: Delete the next word
            KEY_ALT_DELETE => {
                if cursor < len {
                    let old_len = len;
                    let start = cursor;
                    while cursor < len && buffer[cursor] == b' ' {
                        cursor += 1;
                    }
                    while cursor < len && buffer[cursor] != b' ' {
                        cursor += 1;
                    }
                    let deleted_count = cursor - start;
                    for i in cursor..len {
                        buffer[i - deleted_count] = buffer[i];
                    }
                    len -= deleted_count;
                    cursor = start;
                    redraw_line(prompt_x, prompt_y, &buffer, len, cursor, old_len, None);
                }
            }

            // Ctrl + U: Clear entire line
            KEY_CTRL_U => {
                if len > 0 {
                    let old_len = len;
                    len = 0;
                    cursor = 0;
                    redraw_line(prompt_x, prompt_y, &buffer, len, cursor, old_len, None);
                }
            }

            // Ctrl + K: Kill line from cursor to end
            KEY_CTRL_K => {
                if cursor < len {
                    let old_len = len;
                    len = cursor;
                    redraw_line(prompt_x, prompt_y, &buffer, len, cursor, old_len, None);
                }
            }

            // Ctrl + W: Delete word backwards
            KEY_CTRL_W => {
                if cursor > 0 {
                    let old_len = len;
                    let mut new_cursor = cursor;
                    // Skip spaces before cursor
                    while new_cursor > 0 && buffer[new_cursor - 1] == b' ' {
                        new_cursor -= 1;
                    }
                    // Skip word characters
                    while new_cursor > 0 && buffer[new_cursor - 1] != b' ' {
                        new_cursor -= 1;
                    }
                    let deleted_count = cursor - new_cursor;
                    for i in cursor..len {
                        buffer[i - deleted_count] = buffer[i];
                    }
                    len -= deleted_count;
                    cursor = new_cursor;
                    redraw_line(prompt_x, prompt_y, &buffer, len, cursor, old_len, None);
                }
            }

            // Shift+Tab: cycle completion backwards is not supported; clear selection.
            KEY_SHIFT_TAB => { selection_anchor = None; }

            // Tab completion
            KEY_TAB => {
                if selection_anchor.is_some() {
                    delete_selection(&mut buffer, &mut len, &mut cursor, &mut selection_anchor);
                }
                complete_input(&mut buffer, &mut len, &mut cursor);
                redraw_line(prompt_x, prompt_y, &buffer, len, cursor, len, None);
            }

            // Printable ASCII characters
            ascii if ascii >= 32 && ascii <= 126 => {
                let ch = ascii as u8;
                let old_len = len;
                delete_selection(&mut buffer, &mut len, &mut cursor, &mut selection_anchor);
                if len < MAX_CMD_LEN - 1 {
                    for i in (cursor..len).rev() {
                        buffer[i + 1] = buffer[i];
                    }
                    buffer[cursor] = ch;
                    cursor += 1;
                    len += 1;
                    redraw_line(prompt_x, prompt_y, &buffer, len, cursor, old_len, None);
                }
            }

            // Ignore unhandled keys
            _ => {}
        }
    }
}

fn input_position(prompt_x: usize, prompt_y: usize, offset: usize) -> Option<(usize, usize)> {
    let (cols, rows) = vga::get_dimensions();
    if cols == 0 || rows == 0 {
        return None;
    }
    let absolute = prompt_x.saturating_add(offset);
    let x = absolute % cols;
    let y = prompt_y.saturating_add(absolute / cols);
    (y < rows).then_some((x, y))
}

fn set_input_cursor(prompt_x: usize, prompt_y: usize, offset: usize) {
    if let Some((x, y)) = input_position(prompt_x, prompt_y, offset) {
        vga::set_cursor(x, y);
    }
}

fn redraw_line(
    prompt_x: usize,
    prompt_y: usize,
    buffer: &[u8],
    len: usize,
    cursor: usize,
    old_len: usize,
    selection_anchor: Option<usize>,
) {
    let draw_len = len.max(old_len);
    for offset in 0..draw_len {
        let Some((x, y)) = input_position(prompt_x, prompt_y, offset) else { break; };
        let ch = if offset < len { buffer[offset] } else { b' ' };
        let selected = match (selection_anchor, offset < len) {
            (Some(anchor), true) => offset >= anchor.min(cursor) && offset < anchor.max(cursor),
            _ => false,
        };
        let color = if selected {
            vga::make_color(Color::Black, Color::LightGray)
        } else {
            vga::make_color(Color::White, Color::Black)
        };
        vga::putchar_at(ch, color, x, y);
    }
    set_input_cursor(prompt_x, prompt_y, cursor);
}

fn delete_selection(
    buffer: &mut [u8; MAX_CMD_LEN],
    len: &mut usize,
    cursor: &mut usize,
    anchor: &mut Option<usize>,
) -> bool {
    let Some(start) = *anchor else { return false; };
    let low = start.min(*cursor);
    let high = start.max(*cursor);
    if low == high { *anchor = None; return false; }
    for i in high..*len { buffer[i - (high - low)] = buffer[i]; }
    *len -= high - low;
    *cursor = low;
    *anchor = None;
    true
}

fn complete_input(buffer: &mut [u8; MAX_CMD_LEN], len: &mut usize, cursor: &mut usize) {
    if *cursor != *len { return; }
    let start = buffer[..*len].iter().rposition(|&b| b == b' ').map_or(0, |i| i + 1);
    let prefix = &buffer[start..*len];
    let mut match_name: Option<alloc::string::String> = None;
    let commands = ["help", "clear", "about", "sysinfo", "free", "uptime", "date", "time", "ls", "cat", "touch", "write", "echo", "mway", "vmm", "reboot"];
    for name in commands.iter() {
        if name.as_bytes().starts_with(prefix) && name.len() > prefix.len() {
            if match_name.is_some() { return; }
            match_name = Some(alloc::string::String::from(*name));
        }
    }
    if let Some(name) = match_name {
        let suffix = &name.as_bytes()[prefix.len()..];
        if *len + suffix.len() < MAX_CMD_LEN {
            buffer[*len..*len + suffix.len()].copy_from_slice(suffix);
            *len += suffix.len();
            *cursor = *len;
        }
    }
}

fn print_prompt() {
    print_colored!(Color::LightGreen, Color::Black, "nyxara");
    print_colored!(Color::LightCyan, Color::Black, "> ");
}
