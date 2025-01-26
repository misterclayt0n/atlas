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
    Selection {
        anchor: TextPosition, // Where it starts.
        active: TextPosition, // Where it is currently.
    },
}

/// Represents a position in the text buffer.
#[derive(Debug, Default, Copy, Clone)]
pub struct TextPosition {
    pub line: usize,
    pub col: usize,
    pub offset: usize, // Linear position in the buffer.
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
            Self::Selection { active, .. } => *active,
        }
    }

    /// Converts cursor position to screen coordinates.
    pub fn to_point(&self, char_width: f32, line_height: f32) -> Point {
        let pos = self.position();
        Point::new(pos.col as f32 * char_width, pos.line as f32 * line_height)
    }

    /// Start selection from current position
    pub fn start_selection(&mut self) {
        if let Self::Normal { position, .. } = *self {
            *self = Self::Selection {
                anchor: position,
                active: position,
            };
        }
    }

    /// Clear any active selection
    pub fn clear_selection(&mut self) {
        if let Self::Selection { active, .. } = *self {
            *self = Self::Normal {
                position: active,
                preferred_column: None,
            };
        }
    }

    //
    // Movement
    //

    pub fn move_left(&mut self, _buffer: &Buffer) -> Option<TextPosition> {
        let current = self.position();

        // Only allow movement within the current line.
        if current.col > 0 {
            Some(TextPosition::new(
                current.line,
                current.col - 1,
                current.offset - 1,
            ))
        } else {
            None
        }
    }

    pub fn move_right(&mut self, buffer: &Buffer) -> Option<TextPosition> {
        let current = self.position();
        let visual_len = buffer.visual_line_length(current.line);

        // Only allow movement within the current line.
        if current.col < visual_len {
            Some(TextPosition::new(
                current.line,
                current.col + 1,
                current.offset + 1,
            ))
        } else {
            None
        }
    }

    pub fn move_up(&mut self, buffer: &Buffer) -> Option<TextPosition> {
        let current = self.position();
        if current.line > 0 {
            let target_col = self.get_preferred_column().unwrap_or(current.col);
            let prev_line_len = buffer.visual_line_length(current.line - 1);
            let new_col = target_col.min(prev_line_len);

            Some(TextPosition::new(
                current.line - 1,
                new_col,
                self.calculate_offset(current.line - 1, new_col, buffer),
            ))
        } else {
            None
        }
    }

    pub fn move_down(&mut self, buffer: &Buffer) -> Option<TextPosition> {
        let current = self.position();
        if current.line < buffer.content.len_lines() - 1 {
            let target_col = self.get_preferred_column().unwrap_or(current.col);
            let next_line_len = buffer.visual_line_length(current.line + 1);
            let new_col = target_col.min(next_line_len);

            Some(TextPosition::new(
                current.line + 1,
                new_col,
                self.calculate_offset(current.line + 1, new_col, buffer)
            ))
        } else {
            None
        }
    }

    pub fn move_to_position(
        &mut self,
        position: TextPosition,
        buffer: &Buffer,
    ) -> Option<TextPosition> {
        // Directly set position with bound checking.
        let clamped_line = position
            .line
            .min(buffer.content.len_lines().saturating_sub(1));
        let line_len = buffer.visual_line_length(clamped_line);
        let clamped_col = position.col.min(line_len);
        let offset = buffer.content.line_to_char(clamped_line) + clamped_col;

        let new_position = TextPosition::new(clamped_line, clamped_col, offset);

        match self {
            Cursor::Normal {
                position: pos,
                preferred_column,
            } => {
                *pos = new_position;
                *preferred_column = Some(new_position.col); // Update preferred column
            }
            Cursor::Selection { active, .. } => {
                *active = new_position;
            }
        }

        Some(new_position)
    }

    //
    // Helpers
    //

    fn get_preferred_column(&self) -> Option<usize> {
        match &self {
            Cursor::Normal {
                preferred_column, ..
            } => *preferred_column,
            _ => None,
        }
    }

    fn calculate_offset(&self, line: usize, col: usize, buffer: &Buffer) -> usize {
        buffer.content.line_to_byte(line) + col
    }
}

impl TextPosition {
    /// Create a new position from line and column.
    pub fn new(line: usize, col: usize, offset: usize) -> Self {
        Self { line, col, offset }
    }
}
