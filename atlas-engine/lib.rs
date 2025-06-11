pub mod buffer;
pub mod cursor;

pub use buffer::Buffer;
pub use cursor::{Cursor, TextPosition};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum VimMode {
    Normal,
    Insert,
    Visual
}

#[derive(Debug, Clone)]
/// Represents possible actions that can be performed in the editor.
pub enum Message {
    Quit,
}
