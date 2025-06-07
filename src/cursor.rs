use iced::Point;

use super::buffer::Buffer;

/// Represents the cursor state in the editor.
#[derive(Clone)]
pub enum Cursor {
    // NOTE: Is this vim-mode agnostic?
    /// Single cursor position
    Normal {
        position: TextPosition,
        preferred_column: Option<usize>, // For maintaining column position during vertical movements.
    },

    /// Text selection with start and end positions.
    _Selection {
        anchor: TextPosition, // Where it starts.
        active: TextPosition, // Where it is currently.
    },
}

/// Represents a position in the text buffer.
#[derive(Debug, Default, Copy, Clone)]
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
        Self::Normal {
            position: TextPosition::default(),
            preferred_column: None,
        }
    }

    /// Get the current position regardless of the cursor type
    pub fn position(&self) -> TextPosition {
        match self {
            Self::Normal { position, .. } => *position,
            Self::_Selection { active, .. } => *active,
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
    pub fn _start_selection(&mut self) {
        if let Self::Normal { position, .. } = *self {
            *self = Self::_Selection {
                anchor: position,
                active: position,
            };
        } else {
            assert!(false, "start_selection called on non normal cursor");
        }
    }

    /// Clear any active selection.
    pub fn _clear_selection(&mut self) {
        if let Self::_Selection { active, .. } = *self {
            *self = Self::Normal {
                position: active,
                preferred_column: None,
            };
        } else {
            assert!(false, "clear_selection called on non normal cursor");
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

    pub fn move_right(&mut self, buffer: &Buffer) -> Option<TextPosition> {
        let cur = self.position();
        buffer.validate_position(&cur);

        if cur.col >= buffer.grapheme_len(cur.line) {
            return None;
        }

        let new_col = cur.col + 1;
        let new_off = buffer.grapheme_col_to_offset(cur.line, new_col);
        let new_pos = TextPosition::new(cur.line, new_col, new_off);

        buffer.validate_position(&new_pos);
        self.set_position(new_pos, true);

        Some(new_pos)
    }

    pub fn move_up(&mut self, buffer: &Buffer) -> Option<TextPosition> {
        let cur = self.position();
        buffer.validate_position(&cur);

        if cur.line == 0 {
            return None;
        }

        let target_line = cur.line - 1;
        let target_col = self.get_preferred_column().unwrap_or(cur.col);
        let new_col = target_col.min(buffer.grapheme_len(target_line));
        let new_off = buffer.grapheme_col_to_offset(target_line, new_col);
        let new_pos = TextPosition::new(cur.line - 1, new_col, new_off);

        buffer.validate_position(&new_pos);
        self.set_position(new_pos, false);

        Some(new_pos)
    }

    pub fn move_down(&mut self, buffer: &Buffer) -> Option<TextPosition> {
        let cur = self.position();
        buffer.validate_position(&cur);

        if cur.line + 1 >= buffer.content.len_lines() {
            return None;
        }

        let target_line = cur.line + 1;
        let target_col = self.get_preferred_column().unwrap_or(cur.col);
        let new_col = target_col.min(buffer.grapheme_len(target_line));
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
        match self {
            Self::Normal {
                position,
                preferred_column,
            } => {
                *position = pos;
                if update_preferred_col {
                    *preferred_column = Some(pos.col)
                }
            }
            Self::_Selection { active, .. } => *active = pos,
        }
    }

    fn get_preferred_column(&self) -> Option<usize> {
        match self {
            Self::Normal {
                preferred_column, ..
            } => *preferred_column,
            _ => None,
        }
    }
}

impl TextPosition {
    /// Create a new position from line and column.
    pub fn new(line: usize, col: usize, offset: usize) -> Self {
        Self { line, col, offset }
    }
}
