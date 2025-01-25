// TODO: Handle fonts a bit better.
// TODO: Add vim mode.
// TODO: Draw the cursor better - This means probably creating a separate independent widget for the cursor itself, not tying it to the editor.
// TODO: File loading/saving
// TODO: Command mode
// TODO: Visual mode
// TODO: Multiple buffer support - Buffer management
// TODO: Status line
// TODO: Line number
// TODO: Syntax Highlighting
// TODO: Split views
// TODO: Multiple cursors - Helix/Zed style
// TODO: LSP
// TODO: Advanced vim features
// TODO?: Completion engine
use engine::workspace::Workspace;
use iced::{Element, Font};

mod engine;
mod ui;

#[derive(Debug, Clone)]
/// Represents possible actions that can be performed in the editor.
pub enum Message {
    TextInput(String),
    CursorMove(CursorMovement),
    InsertChar(char),
    Backspace,
    Delete, // Delete key
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
        let buffer_name = &self.workspace.active_window().editor.buffer.name;

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
                window.editor.buffer.content = text.into();
            }
            Message::CursorMove(movement) => {
                self.workspace
                    .active_window_mut()
                    .editor_mut()
                    .move_cursor(movement);
            }
            Message::InsertChar(c) => {
                let window = self.workspace.active_window_mut();
                let pos = window.editor.cursor.position();
                window.editor.buffer.insert_char(pos.offset, c);
                window.editor_mut().move_cursor(CursorMovement::Right);
            }
            Message::Backspace => {
                let window = self.workspace.active_window_mut();
                let pos = window.editor.cursor.position();
                if pos.offset > 0 {
                    window.editor.buffer.backspace(pos.offset);
                    window.editor_mut().move_cursor(CursorMovement::Left);
                }
            }
            Message::Delete => {
                let window = self.workspace.active_window_mut();
                let pos = window.editor.cursor.position();
                window.editor.buffer.remove_char(pos.offset);
                // Cursor stays in place for delete.
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
