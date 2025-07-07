pub mod buffer;
pub mod cursor;
pub mod multi_cursor;

pub use buffer::Buffer;
pub use cursor::{Cursor, TextPosition};
use iced::widget::pane_grid::{self, Pane};
pub use multi_cursor::MultiCursor;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum EditorMode {
    Normal,
    Insert,
    Visual
}

#[derive(Debug, Clone)]
/// Represents possible actions that can be performed in the editor.
pub enum Message {
    SplitVertical,
    SplitHorizontal,
    PaneClicked(Pane),
    Dragged(pane_grid::DragEvent),
    Resized(pane_grid::ResizeEvent),
    CloseSplit,
    Quit,
}
