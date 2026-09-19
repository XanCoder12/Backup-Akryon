pub const MAX_LINE_LEN: usize = 70;

#[derive(Clone)]
pub struct LineEditor {
    buffer: [u8; MAX_LINE_LEN],
    len: usize,
    cursor: usize,
    selection_anchor: Option<usize>,
}

impl LineEditor {
    pub const fn new() -> Self {
        Self {
            buffer: [0; MAX_LINE_LEN],
            len: 0,
            cursor: 0,
            selection_anchor: None,
        }
    }

    pub fn set_line(&mut self, line: &[u8]) {
        self.len = line.len().min(MAX_LINE_LEN);
        self.buffer[..self.len].copy_from_slice(&line[..self.len]);
        self.cursor = self.len;
        self.selection_anchor = None;
    }

    pub fn clear(&mut self) {
        self.len = 0;
        self.cursor = 0;
        self.selection_anchor = None;
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.buffer[..self.len]
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn cursor(&self) -> usize {
        self.cursor
    }

    pub fn selection(&self) -> Option<(usize, usize)> {
        let anchor = self.selection_anchor?;
        let start = anchor.min(self.cursor);
        let end = anchor.max(self.cursor);
        (start != end).then_some((start, end))
    }

    pub fn clear_selection(&mut self) {
        self.selection_anchor = None;
    }

    pub fn move_left(&mut self) {
        self.cursor = self.cursor.saturating_sub(1);
        self.clear_selection();
    }

    pub fn move_right(&mut self) {
        self.cursor = (self.cursor + 1).min(self.len);
        self.clear_selection();
    }

    pub fn move_home(&mut self) {
        self.cursor = 0;
        self.clear_selection();
    }

    pub fn move_end(&mut self) {
        self.cursor = self.len;
        self.clear_selection();
    }

    pub fn move_word_left(&mut self) {
        while self.cursor > 0 && self.buffer[self.cursor - 1] == b' ' {
            self.cursor -= 1;
        }
        while self.cursor > 0 && self.buffer[self.cursor - 1] != b' ' {
            self.cursor -= 1;
        }
        self.clear_selection();
    }

    pub fn move_word_right(&mut self) {
        while self.cursor < self.len && self.buffer[self.cursor] != b' ' {
            self.cursor += 1;
        }
        while self.cursor < self.len && self.buffer[self.cursor] == b' ' {
            self.cursor += 1;
        }
        self.clear_selection();
    }

    fn begin_selection(&mut self) {
        if self.selection_anchor.is_none() {
            self.selection_anchor = Some(self.cursor);
        }
    }

    pub fn extend_left(&mut self) {
        self.begin_selection();
        self.cursor = self.cursor.saturating_sub(1);
    }

    pub fn extend_right(&mut self) {
        self.begin_selection();
        self.cursor = (self.cursor + 1).min(self.len);
    }

    pub fn extend_home(&mut self) {
        self.begin_selection();
        self.cursor = 0;
    }

    pub fn extend_end(&mut self) {
        self.begin_selection();
        self.cursor = self.len;
    }

    pub fn delete_selection(&mut self) -> bool {
        let Some((start, end)) = self.selection() else {
            self.clear_selection();
            return false;
        };
        for i in end..self.len {
            self.buffer[i - (end - start)] = self.buffer[i];
        }
        self.len -= end - start;
        self.cursor = start;
        self.clear_selection();
        true
    }

    pub fn backspace(&mut self) -> bool {
        if self.delete_selection() {
            return true;
        }
        if self.cursor == 0 {
            return false;
        }
        for i in self.cursor..self.len {
            self.buffer[i - 1] = self.buffer[i];
        }
        self.cursor -= 1;
        self.len -= 1;
        true
    }

    pub fn delete_char(&mut self) -> bool {
        if self.delete_selection() {
            return true;
        }
        if self.cursor == self.len {
            return false;
        }
        for i in (self.cursor + 1)..self.len {
            self.buffer[i - 1] = self.buffer[i];
        }
        self.len -= 1;
        true
    }

    pub fn delete_word_backward(&mut self) -> bool {
        if self.delete_selection() {
            return true;
        }
        if self.cursor == 0 {
            return false;
        }
        let end = self.cursor;
        while self.cursor > 0 && self.buffer[self.cursor - 1] == b' ' {
            self.cursor -= 1;
        }
        while self.cursor > 0 && self.buffer[self.cursor - 1] != b' ' {
            self.cursor -= 1;
        }
        for i in end..self.len {
            self.buffer[i - (end - self.cursor)] = self.buffer[i];
        }
        self.len -= end - self.cursor;
        true
    }

    pub fn delete_word_forward(&mut self) -> bool {
        if self.delete_selection() {
            return true;
        }
        if self.cursor == self.len {
            return false;
        }
        let start = self.cursor;
        while self.cursor < self.len && self.buffer[self.cursor] == b' ' {
            self.cursor += 1;
        }
        while self.cursor < self.len && self.buffer[self.cursor] != b' ' {
            self.cursor += 1;
        }
        for i in self.cursor..self.len {
            self.buffer[i - (self.cursor - start)] = self.buffer[i];
        }
        self.len -= self.cursor - start;
        self.cursor = start;
        true
    }

    pub fn kill_to_end(&mut self) -> bool {
        if self.cursor == self.len {
            return false;
        }
        self.len = self.cursor;
        self.clear_selection();
        true
    }

    pub fn insert(&mut self, byte: u8) -> bool {
        self.delete_selection();
        if self.len == MAX_LINE_LEN {
            return false;
        }
        for i in (self.cursor..self.len).rev() {
            self.buffer[i + 1] = self.buffer[i];
        }
        self.buffer[self.cursor] = byte;
        self.cursor += 1;
        self.len += 1;
        true
    }
}

#[cfg(test)]
mod tests {
    use super::{LineEditor, MAX_LINE_LEN};

    fn editor(line: &[u8]) -> LineEditor {
        let mut editor = LineEditor::new();
        editor.set_line(line);
        editor
    }

    #[test]
    fn inserts_at_cursor_and_backspaces() {
        let mut editor = editor(b"helo");
        editor.move_left();
        assert!(editor.insert(b'l'));
        assert_eq!(editor.as_bytes(), b"hello");
        assert!(editor.backspace());
        assert_eq!(editor.as_bytes(), b"helo");
    }

    #[test]
    fn moves_by_word() {
        let mut editor = editor(b"hello world again");
        editor.move_home();
        editor.move_word_right();
        assert_eq!(editor.cursor(), 6);
        editor.move_word_right();
        assert_eq!(editor.cursor(), 12);
        editor.move_word_left();
        assert_eq!(editor.cursor(), 6);
    }

    #[test]
    fn selection_is_deleted_or_replaced() {
        let mut editor = editor(b"hello world");
        editor.move_home();
        editor.extend_end();
        assert_eq!(editor.selection(), Some((0, 11)));
        assert!(editor.insert(b'x'));
        assert_eq!(editor.as_bytes(), b"x");
    }

    #[test]
    fn deletes_words_in_both_directions() {
        let mut editor = editor(b"hello world again");
        editor.move_home();
        assert!(editor.delete_word_forward());
        assert_eq!(editor.as_bytes(), b" world again");
        editor.move_end();
        assert!(editor.delete_word_backward());
        assert_eq!(editor.as_bytes(), b" world ");
    }

    #[test]
    fn clamps_lines_to_editor_capacity() {
        let mut editor = editor(&[b'x'; MAX_LINE_LEN + 10]);
        assert_eq!(editor.len(), MAX_LINE_LEN);
        assert!(!editor.insert(b'y'));
    }

    #[test]
    fn supports_navigation_and_deletion_commands() {
        let mut editor = editor(b"abc def");
        editor.move_home();
        editor.extend_right();
        editor.extend_home();
        editor.extend_end();
        editor.move_home();
        editor.move_right();
        editor.extend_left();
        assert_eq!(editor.selection(), Some((0, 1)));
        assert!(editor.delete_char());
        assert_eq!(editor.as_bytes(), b"bc def");
        editor.move_end();
        assert!(editor.kill_to_end() == false);
        editor.clear();
        assert!(editor.as_bytes().is_empty());
    }
}
