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
        }
    }

    /// Clear any active selection.
    pub fn _clear_selection(&mut self) {
        if let Self::_Selection { active, .. } = *self {
            *self = Self::Normal {
                position: active,
                preferred_column: None,
            };
        }
    }

    //
    // Movement
    //

    pub fn move_left(&mut self, buffer: &Buffer) -> Option<TextPosition> {
        let cur = self.position();
        if cur.col == 0 {
            return None;
        }

        let new_col = cur.col - 1;
        let new_off = buffer.grapheme_col_to_offset(cur.line, new_col);
        let new_pos = TextPosition::new(cur.line, new_col, new_off);

        self.set_position(new_pos, true);

        Some(new_pos)
    }

    pub fn move_right(&mut self, buffer: &Buffer) -> Option<TextPosition> {
        let cur = self.position();
        if cur.col >= buffer.grapheme_len(cur.line) {
            return None;
        }

        let new_col = cur.col + 1;
        let new_off = buffer.grapheme_col_to_offset(cur.line, new_col);
        let new_pos = TextPosition::new(cur.line, new_col, new_off);

        self.set_position(new_pos, true);

        Some(new_pos)
    }

    pub fn move_up(&mut self, buffer: &Buffer) -> Option<TextPosition> {
        let cur = self.position();
        if cur.line == 0 {
            return None;
        }

        let target_col = self.get_preferred_column().unwrap_or(cur.col);
        let new_col = target_col.min(buffer.grapheme_len(cur.line - 1));
        let new_off = buffer.grapheme_col_to_offset(cur.line - 1, new_col);
        let new_pos = TextPosition::new(cur.line - 1, new_col, new_off);

        self.set_position(new_pos, false);

        Some(new_pos)
    }

    pub fn move_down(&mut self, buffer: &Buffer) -> Option<TextPosition> {
        let cur = self.position();
        if cur.line + 1 >= buffer.content.len_lines() {
            return None;
        }

        let target_col = self.get_preferred_column().unwrap_or(cur.col);
        let new_col = target_col.min(buffer.grapheme_len(cur.line + 1));
        let new_off = buffer.grapheme_col_to_offset(cur.line + 1, new_col);
        let new_pos = TextPosition::new(cur.line + 1, new_col, new_off);
        self.set_position(new_pos, false);

        Some(new_pos)
    }

    pub fn move_to_position(&mut self, pos: TextPosition, buffer: &Buffer) -> Option<TextPosition> {
        assert!(
            pos.line < buffer.content.len_lines(),
            "Line index out of bounds: {} >= {}",
            pos.line,
            buffer.content.len_lines()
        );

        let line = pos.line.min(buffer.content.len_lines().saturating_sub(1));
        let col = pos.col.min(buffer.grapheme_len(line));
        let off = buffer.grapheme_col_to_offset(line, col);
        let new_pos = TextPosition::new(line, col, off);

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
