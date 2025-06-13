use iced::Point;

use crate::VimMode;
use super::buffer::Buffer;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cursor {
    state: CursorState,
    preferred_column: Option<usize>, // Global preferred column for all modes.
}

/// Represents the cursor state in the editor.
#[derive(Debug, Clone, PartialEq, Eq)]
enum CursorState {
    /// Single cursor position.
    Normal {
        position: TextPosition,
    },

    /// Text selection with start and end positions.
    Selection {
        anchor: TextPosition, // Where it starts.
        active: TextPosition, // Where it is currently.
    },
}

/// Represents a position in the text buffer.
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
pub struct TextPosition {
    pub line: usize,
    pub col: usize,
    pub offset: usize, // Linear position in the buffer (character count from start).
}

#[derive(PartialEq)]
enum CharClass {
    Whitespace,
    Word,
    Punctuation,
}

fn get_char_class(c: char, big_word: bool) -> CharClass {
    if c.is_whitespace() {
        CharClass::Whitespace
    } else if big_word {
        CharClass::Word // All non-whitespace is a WORD.
    } else if c.is_alphanumeric() || c == '_' {
        CharClass::Word
    } else {
        CharClass::Punctuation
    }
}

impl Cursor {
    pub fn new() -> Self {
        // This never starts with selection.
        Self {
            state: CursorState::Normal { position: TextPosition::default() },
            preferred_column: None,
        }
    }

    /// Get the current position regardless of the cursor type
    pub fn position(&self) -> TextPosition {
        match &self.state {
            CursorState::Normal { position } => *position,
            CursorState::Selection { active, .. } => *active,
        }
    }

    /// Converts cursor position to screen coordinates.
    pub fn _to_point(&self, char_width: f32, line_height: f32) -> Point {
        assert!(
            char_width > 0.0,
            "Character width {} must be positive",
            char_width
        );
        assert!(
            line_height > 0.0,
            "Line height {} must be positive",
            line_height
        );

        let pos = self.position();
        Point::new(pos.col as f32 * char_width, pos.line as f32 * line_height)
    }

    /// Start selection from current position.
    pub fn start_selection(&mut self) {
        if let CursorState::Normal { position } = self.state {
            self.state = CursorState::Selection {
                anchor: position,
                active: position,
            };
        } else {
            assert!(false, "start_selection called on non normal cursor");
        }
    }

    /// Clear any active selection.
    pub fn clear_selection(&mut self) {
        if let CursorState::Selection { active, .. } = self.state {
            self.state = CursorState::Normal { position: active };
        } else {
            assert!(false, "clear_selection called on non normal cursor");
        }
    }

    /// Check if cursor is in selection mode.
    pub fn has_selection(&self) -> bool {
        matches!(&self.state, CursorState::Selection { .. })
    }

    /// Get selection range if in selection mode.
    pub fn get_selection(&self) -> Option<(TextPosition, TextPosition)> {
        if let CursorState::Selection { anchor, active } = &self.state {
            // NOTE: In visual mode, selection is always inclusive of both anchor and active positions.
            // That's why we adjust the end position to include the character under the cursor.
            let (start, mut end) = if anchor.offset <= active.offset {
                (*anchor, *active)
            } else {
                (*active, *anchor)
            };

            // Make the selection inclusive by extending the end position by 1.
            // This ensures the character under the cursor is always highlighted.
            end.col += 1;
            end.offset += 1;

            Some((start, end))
        } else {
            None
        }
    }

    //
    // Movement
    //

    pub fn move_left(&mut self, buffer: &Buffer) -> Option<TextPosition> {
        let cur = self.position();
        buffer.validate_position(&cur);

        if cur.col == 0 {
            return None;
        }

        let new_col = cur.col - 1;
        let new_off = buffer.grapheme_col_to_offset(cur.line, new_col);
        let new_pos = TextPosition::new(cur.line, new_col, new_off);

        buffer.validate_position(&new_pos);

        self.set_position(new_pos, true);

        Some(new_pos)
    }

    pub fn move_right(&mut self, buffer: &Buffer, vim_mode: &VimMode) -> Option<TextPosition> {
        let cur = self.position();
        buffer.validate_position(&cur);

        // In Normal mode, cursor can't go past the last character
        // In Insert mode, cursor can go one position past the last character
        let max_col = self.get_max_col(vim_mode, buffer, cur.line);

        if cur.col >= max_col {
            return None;
        }

        let new_col = cur.col + 1;
        let new_off = buffer.grapheme_col_to_offset(cur.line, new_col);
        let new_pos = TextPosition::new(cur.line, new_col, new_off);

        buffer.validate_position(&new_pos);
        self.set_position(new_pos, true);

        Some(new_pos)
    }

    pub fn move_up(&mut self, buffer: &Buffer, vim_mode: &VimMode) -> Option<TextPosition> {
        let cur = self.position();
        buffer.validate_position(&cur);

        if cur.line == 0 {
            return None;
        }

        let target_line = cur.line - 1;
        let target_col = self.preferred_column.unwrap_or(cur.col);

        let max_col = self.get_max_col(vim_mode, buffer, target_line);
        let new_col = target_col.min(max_col);
        let new_off = buffer.grapheme_col_to_offset(target_line, new_col);
        let new_pos = TextPosition::new(cur.line - 1, new_col, new_off);

        buffer.validate_position(&new_pos);
        self.set_position(new_pos, false);

        Some(new_pos)
    }

    pub fn move_down(&mut self, buffer: &Buffer, vim_mode: &VimMode) -> Option<TextPosition> {
        let cur = self.position();
        buffer.validate_position(&cur);

        if cur.line + 1 >= buffer.content.len_lines() {
            return None;
        }

        let target_line = cur.line + 1;
        let target_col = self.preferred_column.unwrap_or(cur.col);

        let max_col = self.get_max_col(vim_mode, buffer, target_line);
        let new_col = target_col.min(max_col);
        let new_off = buffer.grapheme_col_to_offset(target_line, new_col);
        let new_pos = TextPosition::new(cur.line + 1, new_col, new_off);

        buffer.validate_position(&new_pos);
        self.set_position(new_pos, false);

        Some(new_pos)
    }

    pub fn move_word_forward(&mut self, buffer: &Buffer, big_word: bool) -> Option<TextPosition> {
        let total_chars = buffer.content.len_chars();
        let cur = self.position();

        buffer.validate_position(&cur);

        let line_start = buffer.content.line_to_char(cur.line);

        assert!(
            line_start <= total_chars,
            "Line start offset {} exceeds total characters {}",
            line_start,
            total_chars
        );

        let mut char_idx = line_start + cur.col;

        // If we're at or beyond the end of the buffer, no movement is possible.
        if char_idx >= total_chars {
            return None;
        }

        // Get the current character and it's class.
        let c = buffer.content.char(char_idx);
        let current_class = get_char_class(c, big_word);

        // Skip over characters of the same class.
        while char_idx < total_chars {
            let c = buffer.content.char(char_idx);
            let class = get_char_class(c, big_word);
            if class == current_class {
                char_idx += 1; // Move to next character.
            } else {
                break;
            }
        }

        // Skip over any whitespace characters.
        while char_idx < total_chars {
            let c = buffer.content.char(char_idx);
            if get_char_class(c, big_word) == CharClass::Whitespace {
                char_idx += 1;
            } else {
                break;
            }
        }

        // If we've reached the end, no valid position is found.
        if char_idx >= total_chars {
            return None;
        }

        // Convert char_idx to TextPosition.
        let new_line = buffer.content.char_to_line(char_idx);
        let line_start = buffer.content.line_to_char(new_line);

        assert!(
            line_start <= total_chars,
            "New line start offset {} exceeds total characters {}",
            line_start,
            total_chars
        );

        let new_col = char_idx - line_start;
        let new_pos = TextPosition::new(new_line, new_col, char_idx);

        buffer.validate_position(&new_pos);
        self.set_position(new_pos, true);

        Some(new_pos)
    }

    pub fn move_word_backward(&mut self, buffer: &Buffer, big_word: bool) -> Option<TextPosition> {
        let total_chars = buffer.content.len_chars();
        let cur = self.position();

        buffer.validate_position(&cur);

        let line_start = buffer.content.line_to_char(cur.line);

        assert!(
            line_start <= total_chars,
            "Line start offset {} exceeds total characters {}",
            line_start,
            total_chars
        );

        let mut char_idx = line_start + cur.col;

        if char_idx == 0 {
            return None;
        }

        // Move the cursor one step back to start looking at the previous character.
        char_idx = char_idx.saturating_sub(1);

        // Get the current character and it's class.
        let c = buffer.content.char(char_idx);
        let current_class = get_char_class(c, big_word);

        // Skip any trailing whitespace.
        while char_idx > 0 {
            let c = buffer.content.char(char_idx);
            let class = get_char_class(c, big_word);
            if class == current_class {
                char_idx = char_idx.saturating_sub(1);
            } else {
                break;
            }
        }

        // Skip all characters that are of the same class.
        while char_idx > 0 {
            let c = buffer.content.char(char_idx);
            let class = get_char_class(c, big_word);
            if class == current_class {
                char_idx = char_idx.saturating_sub(1);
            } else {
                // Stop at the boundary between different character classes.
                char_idx += 1;
                break;
            }
        }

        while char_idx > 0 {
            let c = buffer.content.char(char_idx);
            let class = get_char_class(c, big_word);
            if class == CharClass::Whitespace {
                char_idx = char_idx.saturating_sub(1);
            } else {
                break;
            }
        }

        let new_line = buffer.content.char_to_line(char_idx);
        let new_line_start = buffer.content.line_to_char(new_line);
        let new_col = char_idx - new_line_start;
        let new_pos = TextPosition::new(new_line, new_col, char_idx);

        buffer.validate_position(&new_pos);
        self.set_position(new_pos, true);

        Some(new_pos)
    }

    pub fn move_word_end(&mut self, buffer: &Buffer, big_word: bool) -> Option<TextPosition> {
        let total_chars = buffer.content.len_chars();
        let cur = self.position();

        buffer.validate_position(&cur);

        let line_start = buffer.content.line_to_char(cur.line);
        let mut char_idx = line_start + cur.col;

        if char_idx >= total_chars {
            return None;
        }

        // Move forward one character if possible.
        if char_idx + 1 < total_chars {
            char_idx += 1;
        } else {
            // We're at the end of the buffer.
            return None;
        }

        // Skip over whitespace.
        while char_idx < total_chars {
            let c = buffer.content.char(char_idx);
            if get_char_class(c, big_word) == CharClass::Whitespace {
                char_idx += 1;
            } else {
                break;
            }
        }

        if char_idx >= total_chars {
            return None;
        }

        let current_class = get_char_class(buffer.content.char(char_idx), big_word);
        let mut last_char_index = char_idx;

        // Move to the end of the current class sequence.
        while char_idx < total_chars {
            let c = buffer.content.char(char_idx);
            if get_char_class(c, big_word) == current_class {
                last_char_index = char_idx;
                char_idx += 1;
            } else {
                break;
            }
        }

        // Convert char index back to TextPosition
        let new_line = buffer.content.char_to_line(last_char_index);
        let new_line_start = buffer.content.line_to_char(new_line);
        let new_col = last_char_index - new_line_start;
        let new_pos = TextPosition::new(new_line, new_col, last_char_index);

        buffer.validate_position(&new_pos);
        self.set_position(new_pos, true);

        Some(new_pos)
    }

    pub fn move_to_position(&mut self, pos: TextPosition, buffer: &Buffer) -> Option<TextPosition> {
        buffer.validate_position(&pos);

        let line = pos.line.min(buffer.content.len_lines().saturating_sub(1));
        let col = pos.col.min(buffer.grapheme_len(line));
        let off = buffer.grapheme_col_to_offset(line, col);
        let new_pos = TextPosition::new(line, col, off);
        buffer.validate_position(&new_pos);

        self.set_position(new_pos, true);

        Some(new_pos)
    }

    //
    // Helpers
    //

    fn set_position(&mut self, pos: TextPosition, update_preferred_col: bool) {
        match &mut self.state {
            CursorState::Normal { position } => {
                *position = pos;
                if update_preferred_col {
                    self.preferred_column = Some(pos.col)
                }
            }
            CursorState::Selection { active, .. } => {
                *active = pos;
                if update_preferred_col {
                    self.preferred_column = Some(pos.col)
                }
            }
        }
    }

    pub fn adjust_for_mode(&mut self, buffer: &Buffer, vim_mode: &VimMode) {
        match vim_mode {
            VimMode::Normal => {
                // Clear selection when entering Normal mode.
                if self.has_selection() {
                    self.clear_selection();
                }

                let cur = self.position();
                let line_len = buffer.grapheme_len(cur.line);
                if line_len > 0 && cur.col >= line_len {
                    // Move cursor back to the last character.
                    let new_col = line_len - 1;
                    let new_off = buffer.grapheme_col_to_offset(cur.line, new_col);
                    let new_pos = TextPosition::new(cur.line, new_col, new_off);
                    self.set_position(new_pos, true);
                }
            }
            VimMode::Insert => {
                // Clear selection when entering Insert mode.
                if self.has_selection() {
                    self.clear_selection();
                }
            }
            VimMode::Visual => {
                // Start selection when entering Visual mode if we don't already have one.
                if !self.has_selection() {
                    self.start_selection();
                }
            }
        }
    }

    fn get_max_col(&self, vim_mode: &VimMode, buffer: &Buffer, target: usize) -> usize {
        match vim_mode {
            VimMode::Normal | VimMode::Visual => {
                let line_len = buffer.grapheme_len(target);
                if line_len == 0 {
                    0
                } else {
                    line_len - 1
                }
            }
            VimMode::Insert => buffer.grapheme_len(target),
        }
    }
}

impl TextPosition {
    /// Create a new position from line and column.
    pub fn new(line: usize, col: usize, offset: usize) -> Self {
        Self { line, col, offset }
    }
}
