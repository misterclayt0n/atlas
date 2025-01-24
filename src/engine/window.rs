use iced::{widget::container, Element, Length, Point, Rectangle};

use crate::{ui::editor::Editor, Message};

use super::buffer::Buffer;

/// Represents a view into a buffer.
/// Manages viewport, scroll position, and cursor location for a specific buffer.
pub struct Window {
    pub buffer: Buffer,
    pub editor: Editor,
    pub _bounds: Rectangle,
    pub _scroll_offset: Point,
}

impl Window {
    pub fn new(buffer: Buffer) -> Self {
        Self {
            editor: Editor::new(buffer.clone()),
            buffer,
            _bounds: Rectangle::default(),
            _scroll_offset: Point::new(0.0, 0.0),
        }
    }

    /// Renders the window's content using the Editor widget.
    pub fn view(&self) -> Element<Message> {
        container(self.editor.clone())
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    pub fn editor_mut(&mut self) -> &mut Editor {
        &mut self.editor
    }
}
