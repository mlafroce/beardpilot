/// The cursor position is stored as a **char index** (not a byte offset) so
/// that multi-byte Unicode characters are handled correctly.
pub struct TextInput {
    /// The text currently in the input box.
    buf: String,
    /// Cursor position measured in *characters* (not bytes).
    cursor: usize,
}

impl TextInput {
    pub fn new() -> Self {
        Self {
            buf: String::new(),
            cursor: 0,
        }
    }

    // ── read-only accessors ────────────────────────────────────────────────

    pub fn as_str(&self) -> &str {
        &self.buf
    }

    /// Cursor position in *characters* from the start of the buffer.
    pub fn cursor(&self) -> usize {
        self.cursor
    }

    // ── editing ────────────────────────────────────────────────────────────

    /// Insert a character at the current cursor position and advance the cursor.
    pub fn insert(&mut self, c: char) {
        let byte_idx = self.char_to_byte(self.cursor);
        self.buf.insert(byte_idx, c);
        self.cursor += 1;
    }

    /// Delete the character **before** the cursor (Backspace behaviour).
    pub fn delete_before(&mut self) {
        if self.cursor == 0 {
            return;
        }
        self.cursor -= 1;
        let byte_idx = self.char_to_byte(self.cursor);
        self.buf.remove(byte_idx);
    }

    /// Delete the character **at** the cursor (Delete key behaviour).
    pub fn delete_after(&mut self) {
        let len = self.buf.chars().count();
        if self.cursor >= len {
            return;
        }
        let byte_idx = self.char_to_byte(self.cursor);
        self.buf.remove(byte_idx);
    }

    pub fn move_left(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
        }
    }

    pub fn move_right(&mut self) {
        let len = self.buf.chars().count();
        if self.cursor < len {
            self.cursor += 1;
        }
    }

    pub fn move_start(&mut self) {
        self.cursor = 0;
    }

    pub fn move_end(&mut self) {
        self.cursor = self.buf.chars().count();
    }

    /// Place the cursor at a column offset relative to the left edge of the
    /// input widget.  `col` is the 0-based column *within the widget*.
    /// Accounts for the `"> "` prefix (2 chars) that the UI prepends.
    pub fn set_cursor_from_click(&mut self, col: u16) {
        // The UI renders "> " then the buffer text; col 0 and 1 are the prompt.
        let text_col = (col as usize).saturating_sub(2);
        let len = self.buf.chars().count();
        self.cursor = text_col.min(len);
    }

    // ── submission ─────────────────────────────────────────────────────────

    /// Return the current buffer contents and reset the input to empty.
    pub fn take(&mut self) -> String {
        self.cursor = 0;
        std::mem::take(&mut self.buf)
    }

    // ── helpers ────────────────────────────────────────────────────────────

    /// Convert a char-index to the corresponding byte offset in `self.buf`.
    fn char_to_byte(&self, char_idx: usize) -> usize {
        self.buf
            .char_indices()
            .nth(char_idx)
            .map(|(b, _)| b)
            .unwrap_or(self.buf.len())
    }
}

impl Default for TextInput {
    fn default() -> Self {
        Self::new()
    }
}
