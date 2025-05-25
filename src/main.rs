// TODO: Iterate by graphemes instead of bytes.
// TODO: Add vim mode.
// TODO: Vim bindings. -> At least the basics.
// TODO: File loading/saving.
// TODO: Command mode.
// TODO: Visual mode.
// TODO: Multiple buffer support - Buffer management.
// TODO: Status line.
// TODO: Line number.
// TODO: Syntax Highlighting.
// TODO: Split views.
// TODO: Multiple cursors - Helix/Zed style.
// TODO: LSP.
// TODO: Advanced vim features.
// TODO?: Completion engine.
use engine::{cursor::TextPosition, workspace::Workspace};
use iced::Element;

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
    Position(TextPosition),
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
                    if pos.col == 0 && pos.line > 0 {
                        // Move cursor to the end of previous line.
                        let prev_line_length =
                            window.editor.buffer.visual_line_length(pos.line - 1);
                        window.editor.buffer.backspace(pos.offset);
                        window.editor_mut().move_cursor(CursorMovement::Position(
                            TextPosition::new(pos.line - 1, prev_line_length, pos.offset - 1),
                        ));
                    } else {
                        // Normal backspace behavior.
                        window.editor.buffer.backspace(pos.offset);
                        window.editor_mut().move_cursor(CursorMovement::Left)
                    }
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

pub struct Iosevka;

impl Iosevka {
	pub const REGULAR: iced::Font = iced::Font {
		style: iced::font::Style::Normal,
		family: iced::font::Family::Name("Iosevka"),
		stretch: iced::font::Stretch::Normal,
		weight: iced::font::Weight::Normal,
	};
}

fn main() -> iced::Result {
    iced::application(Atlas::title, Atlas::update, Atlas::view)
    	.font(include_bytes!("../fonts/iosevka-regular.ttf"))
        .default_font(Iosevka::REGULAR)
        .run()
}
