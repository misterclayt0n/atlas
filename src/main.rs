// TODO: Add some lerp animation to scrolling (minimal stuff).
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
// FIX: Scrolling vertically (from bottom to top, it gets "glued").

use cursor::TextPosition;
use editor::Editor;
use iced::Element;

mod buffer;
mod cursor;
mod editor;

#[derive(Debug, Clone)]
/// Represents possible actions that can be performed in the editor.
pub enum Message {
    TextInput(String),
    InsertChar(char),
    Backspace,
    Delete, // Delete key
    Quit,
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
    editors: Vec<Editor>,
    active_editor: usize,
}

impl Default for Atlas {
    fn default() -> Self {
        Self {
            editors: vec![Editor::new()],
            active_editor: 0,
        }
    }
}

impl Atlas {
    /// Generates the window title based on the active buffer
    fn title(&self) -> String {
        format!("Atlas - {}", self.editors[self.active_editor].buffer.name)
    }

    /// Handles all editor actions and updates state accordingly
    fn update(&mut self, message: Message) {
        let editor = &mut self.editors[self.active_editor];

        match message {
            Message::TextInput(text) => editor.buffer.content = text.into(),
            Message::InsertChar(c) => {
                let pos = editor.cursor.position();
                editor.buffer.insert_char(pos.offset, c);
                editor.move_cursor(CursorMovement::Right);
            }
            Message::Backspace => {
                let pos = editor.cursor.position();
                if pos.offset > 0 {
                    if pos.col == 0 && pos.line > 0 {
                        // Move cursor to the end of previous line.
                        let prev_line_length = editor.buffer.visual_line_length(pos.line - 1);
                        editor.buffer.backspace(pos.offset);
                        editor.move_cursor(CursorMovement::Position(TextPosition::new(
                            pos.line - 1,
                            prev_line_length,
                            pos.offset - 1,
                        )));
                    } else {
                        // Normal backspace behavior.
                        editor.buffer.backspace(pos.offset);
                        editor.move_cursor(CursorMovement::Left)
                    }
                }
            }
            Message::Delete => {
                // Cursor stays in place for delete.
                let pos = editor.cursor.position();
                editor.buffer.delete(pos.offset);
            }
            Message::Quit => {
                std::process::exit(0);
            }
        }
    }

    /// Renders the entire editor interface
    fn view(&self) -> Element<Message> {
        // Render
        self.editors[self.active_editor].clone().into()
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
