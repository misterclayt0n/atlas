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
#[derive(Default, Copy, Clone)]
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

    pub fn move_left(&mut self, buffer: &Buffer) -> Option<TextPosition> {
        let current = self.position();

        if current.col > 0 {
            // Move left in the current line.
            Some(TextPosition::new(
                current.line,
                current.col - 1,
                current.offset - 1,
            ))
        } else if current.line > 0 {
            // Move to end of previous line.
            // TODO: Remove this behavior.
            let prev_line = buffer.content.line(current.line - 1);
            Some(TextPosition::new(
                current.line - 1,
                prev_line.len_chars(),
                current.offset - 1,
            ))
        } else {
            None
        }
    }

    pub fn move_right(&mut self, buffer: &Buffer) -> Option<TextPosition> {
        let current = self.position();
        let visual_len = buffer.visual_line_length(current.line);

        if current.col < visual_len {
            // Move right within the current line.
            Some(TextPosition::new(
                current.line,
                current.col + 1,
                current.offset + 1,
            ))
        } else if current.line < buffer.content.len_lines() - 1 {
            // Move to the start of next line.
            // TODO: Remove this behavior.
            Some(TextPosition::new(
                current.line + 1,
                0,
                buffer.content.line_to_byte(current.line + 1),
            ))
        } else {
            None
        }
    }

    pub fn move_up(&mut self, buffer: &Buffer) -> Option<TextPosition> {
        let current = self.position();
        if current.line > 0 {
            let target_col = if let Some(preferred) = self.get_preferred_column() {
                preferred
            } else {
                current.col
            };

            let prev_line_len = buffer.content.line(current.line - 1).len_chars();
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
            let target_col = if let Some(preferred) = self.get_preferred_column() {
                preferred
            } else {
                current.col
            };

            let next_line_len = buffer.content.line(current.line + 1).len_chars();
            let new_col = target_col.min(next_line_len);

            Some(TextPosition::new(
                current.line + 1,
                new_col,
                buffer.content.line_to_byte(current.line + 1) + new_col,
            ))
        } else {
            None
        }
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
