use iced::{widget::container, Element, Length, Point, Rectangle};

use crate::{ui::editor::create_editor, Message};

use super::buffer::Buffer;

/// Represents a view into a buffer.
/// Manages viewport, scroll position, and cursor location for a specific buffer.
pub struct Window {
    pub buffer: Buffer,
    pub _bounds: Rectangle,
    pub _scroll_offset: Point,
}

impl Window {
    pub fn new(buffer: Buffer) -> Self {
        Self {
            buffer,
            _bounds: Rectangle::default(),
            _scroll_offset: Point::new(0.0, 0.0),
        }
    }

    /// Renders the window's content using the Editor widget.
    pub fn view(&self) -> Element<Message> {
        container(create_editor(&self.buffer.content))
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}
