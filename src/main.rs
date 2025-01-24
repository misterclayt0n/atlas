use engine::workspace::Workspace;
use iced::{Element, Font};

mod engine;
mod ui;

#[derive(Debug, Clone)]
/// Represents possible actions that can be performed in the editor.
pub enum Message {
    TextInput(String),
    CursorMove(CursorMovement),
}

#[derive(Debug, Clone)]
pub enum CursorMovement {
    Up,
    Down,
    Left,
    Right,
    // TODO: Add more movement.
}

/// Main application structure.
/// Manages the overall editor state and handles high-level operations.
pub struct Atlas {
    workspace: Workspace,
}

impl Default for Atlas {
    fn default() -> Self {
        Self {
            workspace: Workspace::new(),
        }
    }
}

impl Atlas {
    /// Generates the window title based on the active buffer
    fn title(&self) -> String {
        let buffer_name = &self.workspace.active_window().buffer.name;

        if buffer_name.is_empty() {
            return "Atlas".to_string();
        }

        format!("Atlas - {}", buffer_name)
    }

    /// Handles all editor actions and updates state accordingly
    fn update(&mut self, message: Message) {
        match message {
            Message::TextInput(text) => {
                let window = self.workspace.active_window_mut();
                window.buffer.content = text.into();
            }
            Message::CursorMove(movement) => {
                self.workspace
                    .active_window_mut()
                    .editor_mut()
                    .move_cursor(movement);
            }
        }
    }

    /// Renders the entire editor interface
    fn view(&self) -> Element<Message> {
        // Render
        self.workspace.view()
    }
}

fn main() -> iced::Result {
    iced::application(Atlas::title, Atlas::update, Atlas::view).default_font(Font::MONOSPACE).run()
}
