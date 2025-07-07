use iced::Point;

use super::buffer::Buffer;
use crate::EditorMode;

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct Cursor {
    anchor: TextPosition,            // Where the selection starts.
    active: TextPosition,            // Where it is currently.
    preferred_column: Option<usize>, // Global preferred column for all modes.
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

/// How the cursor move should behave.
#[derive(Default, Clone, Copy)]
pub struct MoveOpts {
    /// If `Some(anchor)` we keep / start a selection from `anchor` to `dest`.
    /// If `None` the selection is collapsed at `dest`.
    pub anchor: Option<TextPosition>,
    pub update_preferred_col: bool,
}

fn get_char_class(c: char, big_word: bool) -> CharClass {
    if c.is_whitespace() {
        CharClass::Whitespace
    } else if big_word || c.is_alphanumeric() || c == '_' {
        CharClass::Word
    } else {
        CharClass::Punctuation
    }
}

impl Cursor {
    pub fn new() -> Self {
        let pos = TextPosition::default();

        Self {
            preferred_column: None,
            anchor: pos,
            active: pos,
        }
    }

    pub fn position(&self) -> TextPosition {
        self.active
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

    pub fn collapse_selection(&mut self) {
        self.anchor = self.active;
    }

    /// A selection exists if the anchor and active positions are different.
    pub fn has_selection(&self) -> bool {
        self.anchor != self.active
    }

    /// Get selection range if in selection mode.
    pub fn get_selection_range(&self) -> (TextPosition, TextPosition) {
        if self.anchor.offset <= self.active.offset {
            (self.anchor, self.active)
        } else {
            (self.active, self.anchor)
        }
    }

    //
    // Movement
    //

    pub fn move_left(&mut self, buffer: &Buffer, editor_mode: &EditorMode) -> Option<TextPosition> {
        let cur = self.position();
        buffer.validate_position(&cur);

        if cur.col == 0 {
            return None;
        }

        let new_col = cur.col - 1;
        let new_off = buffer.grapheme_col_to_offset(cur.line, new_col);
        let new_pos = TextPosition::new(cur.line, new_col, new_off);

        buffer.validate_position(&new_pos);

        let keep_anchor = matches!(editor_mode, EditorMode::Visual);
        self.move_to(
            new_pos,
            MoveOpts {
                anchor: if keep_anchor { Some(self.anchor) } else { None },
                update_preferred_col: true,
            },
            buffer,
        );

        Some(new_pos)
    }

    pub fn move_right(
        &mut self,
        buffer: &Buffer,
        editor_mode: &EditorMode,
    ) -> Option<TextPosition> {
        let cur = self.position();
        buffer.validate_position(&cur);

        // In Normal mode, cursor can't go past the last character
        // In Insert mode, cursor can go one position past the last character
        let max_col = self.get_max_col(editor_mode, buffer, cur.line);

        if cur.col >= max_col {
            return None;
        }

        let new_col = cur.col + 1;
        let new_off = buffer.grapheme_col_to_offset(cur.line, new_col);
        let new_pos = TextPosition::new(cur.line, new_col, new_off);

        buffer.validate_position(&new_pos);
        let keep_anchor = matches!(editor_mode, EditorMode::Visual);
        self.move_to(
            new_pos,
            MoveOpts {
                anchor: if keep_anchor { Some(self.anchor) } else { None },
                update_preferred_col: true,
            },
            buffer,
        );

        Some(new_pos)
    }

    pub fn move_up(&mut self, buffer: &Buffer, editor_mode: &EditorMode) -> Option<TextPosition> {
        let cur = self.position();
        buffer.validate_position(&cur);

        if cur.line == 0 {
            return None;
        }

        let target_line = cur.line - 1;
        let target_col = self.preferred_column.unwrap_or(cur.col);

        let max_col = self.get_max_col(editor_mode, buffer, target_line);
        let new_col = target_col.min(max_col);
        let new_off = buffer.grapheme_col_to_offset(target_line, new_col);
        let new_pos = TextPosition::new(cur.line - 1, new_col, new_off);

        buffer.validate_position(&new_pos);
        let keep_anchor = matches!(editor_mode, EditorMode::Visual);
        self.move_to(
            new_pos,
            MoveOpts {
                anchor: if keep_anchor { Some(self.anchor) } else { None },
                update_preferred_col: false,
            },
            buffer,
        );

        Some(new_pos)
    }

    pub fn move_down(&mut self, buffer: &Buffer, editor_mode: &EditorMode) -> Option<TextPosition> {
        let cur = self.position();
        buffer.validate_position(&cur);

        if cur.line + 1 >= buffer.content.len_lines() {
            return None;
        }

        let target_line = cur.line + 1;
        let target_col = self.preferred_column.unwrap_or(cur.col);

        let max_col = self.get_max_col(editor_mode, buffer, target_line);
        let new_col = target_col.min(max_col);
        let new_off = buffer.grapheme_col_to_offset(target_line, new_col);
        let new_pos = TextPosition::new(cur.line + 1, new_col, new_off);

        buffer.validate_position(&new_pos);
        let keep_anchor = matches!(editor_mode, EditorMode::Visual);
        self.move_to(
            new_pos,
            MoveOpts {
                anchor: if keep_anchor { Some(self.anchor) } else { None },
                update_preferred_col: false,
            },
            buffer,
        );

        Some(new_pos)
    }

    pub fn move_word_forward(&mut self, buffer: &Buffer, big_word: bool, editor_mode: &EditorMode) -> Option<TextPosition> {
        let total_chars = buffer.content.len_chars();
        let initial_pos = self.position();

        buffer.validate_position(&initial_pos);

        let line_start = buffer.content.line_to_char(initial_pos.line);

        assert!(
            line_start <= total_chars,
            "Line start offset {} exceeds total characters {}",
            line_start,
            total_chars
        );

        let mut char_idx = line_start + initial_pos.col;

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
        
        let keep_anchor = matches!(editor_mode, EditorMode::Visual);
        self.move_to(
            new_pos,
            MoveOpts {
                anchor: if keep_anchor { Some(self.anchor) } else { Some(initial_pos) },
                update_preferred_col: true,
            },
            buffer,
        );

        Some(new_pos)
    }

    pub fn move_word_backward(&mut self, buffer: &Buffer, big_word: bool, editor_mode: &EditorMode) -> Option<TextPosition> {
        let total_chars = buffer.content.len_chars();
        let initial_pos = self.position();

        buffer.validate_position(&initial_pos);

        let line_start = buffer.content.line_to_char(initial_pos.line);

        assert!(
            line_start <= total_chars,
            "Line start offset {} exceeds total characters {}",
            line_start,
            total_chars
        );

        let mut char_idx = line_start + initial_pos.col;

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
        let keep_anchor = matches!(editor_mode, EditorMode::Visual);
        self.move_to(
            new_pos,
            MoveOpts {
                anchor: if keep_anchor { Some(self.anchor) } else { Some(initial_pos) },
                update_preferred_col: true,
            },
            buffer,
        );

        Some(new_pos)
    }

    pub fn move_word_end(&mut self, buffer: &Buffer, big_word: bool, editor_mode: &EditorMode) -> Option<TextPosition> {
        let total_chars = buffer.content.len_chars();
        let initial_pos = self.position();

        buffer.validate_position(&initial_pos);

        let line_start = buffer.content.line_to_char(initial_pos.line);
        let mut char_idx = line_start + initial_pos.col;

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
        let keep_anchor = matches!(editor_mode, EditorMode::Visual);
        self.move_to(
            new_pos,
            MoveOpts {
                anchor: if keep_anchor { Some(self.anchor) } else { Some(initial_pos) },
                update_preferred_col: true,
            },
            buffer,
        );

        Some(new_pos)
    }

    /// Move the cursor to `dest`, optionally extend / collapse selection and update `preferred_col`.
    ///
    /// Returns the clamped position that was finally reached (or `None` if the move is impossible - e.g.
    /// off the left edge of the buffer).
    pub fn move_to(
        &mut self,
        dest: TextPosition,
        opts: MoveOpts,
        buffer: &Buffer,
    ) -> Option<TextPosition> {
        buffer.validate_position(&dest);

        let line = dest.line.min(buffer.content.len_lines().saturating_sub(1));
        let col = dest.col.min(buffer.grapheme_len(line));
        let off = buffer.grapheme_col_to_offset(line, col);
        let dest = TextPosition::new(line, col, off);
        buffer.validate_position(&dest);

        self.active = dest;
        self.anchor = opts.anchor.unwrap_or(dest);
        if opts.update_preferred_col {
            self.preferred_column = Some(dest.col);
        }

        Some(dest)
    }

    //
    // Helpers
    //

    pub fn adjust_for_mode(&mut self, buffer: &Buffer, editor_mode: &EditorMode) {
        match editor_mode {
            EditorMode::Normal => {
                let cur = self.position();
                let line_len = buffer.grapheme_len(cur.line);
                if line_len > 0 && cur.col >= line_len {
                    // Move cursor back to the last character.
                    let new_col = line_len - 1;
                    let new_off = buffer.grapheme_col_to_offset(cur.line, new_col);
                    let new_pos = TextPosition::new(cur.line, new_col, new_off);
                    self.move_to(
                        new_pos,
                        MoveOpts {
                            anchor: None,
                            update_preferred_col: true,
                        },
                        buffer,
                    );
                }
            }
            // We only care for Normal mode here.
            _ => {}
        }
    }

    fn get_max_col(&self, editor_mode: &EditorMode, buffer: &Buffer, target: usize) -> usize {
        match editor_mode {
            EditorMode::Normal | EditorMode::Visual => {
                let line_len = buffer.grapheme_len(target);
                if line_len == 0 {
                    0
                } else {
                    line_len - 1
                }
            }
            EditorMode::Insert => buffer.grapheme_len(target),
        }
    }
}

impl TextPosition {
    /// Create a new position from line and column.
    pub fn new(line: usize, col: usize, offset: usize) -> Self {
        Self { line, col, offset }
    }
}
