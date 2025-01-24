use iced::Point;

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
}

impl TextPosition {
    /// Create a new position from line and column.
    pub fn new(line: usize, col: usize, offset: usize) -> Self {
        Self { line, col, offset }
    }
}
