use iced::Element;

use crate::Message;

use super::{buffer::Buffer, window::Window};

/// Top-level structure that manages multiple windows and buffers.
/// Handles window layout and active window selection.
pub struct Workspace {
    pub windows: Vec<Window>,
    pub active_window: usize,
}

impl Workspace {
    pub fn new() -> Self {
        // Create initial window with empty buffer.
        let initial_window = Window::new(Buffer::new("Some amazing content", "Initial Buffer")); // MOCKED.
        Self {
            windows: vec![initial_window],
            active_window: 0,
        }
    }

    pub fn active_window(&self) -> &Window {
        &self.windows[self.active_window]
    }

    pub fn active_window_mut(&mut self) -> &mut Window {
        &mut self.windows[self.active_window]
    }

    pub fn view(&self) -> Element<Message> {
        // For now, just show the active window.
        self.active_window().view()
    }
}
