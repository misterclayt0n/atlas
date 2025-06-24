// TODO: Vim operators -> "dd" is a good start but we should stop there and move on for now.
// TODO: File loading/saving.
// TODO: Command mode.
// TODO: Multiple buffer support - Buffer management.
// TODO: Multiple windows - Split view (horizontal/vertical), should be infinite splits btw.
// TODO: Status line.
// TODO: Line number.
// TODO: Syntax Highlighting.
// TODO: Split views.
// TODO: Multiple cursors - Helix/Zed style.
// TODO: LSP.
// TODO: Advanced vim features.
// TODO?: Completion engine.

use atlas_engine::Message;
use atlas_widgets::editor::Editor;
use iced::{Element, widget::Row};

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
        format!(
            "Atlas - {}",
            self.editors[self.active_editor].buffer.borrow().name
        )
    }

    /// Handles all editor actions and updates state accordingly
    fn update(&mut self, message: Message) {
        match message {
            Message::SplitHorizontal => {
                let new_editor = self.editors[self.active_editor].clone();
                self.editors.push(new_editor);
                self.active_editor = self.editors.len() - 1;
                self.editors[self.active_editor].is_focused = true;
            }
            Message::Quit => {
                std::process::exit(0);
            }
            Message::FocusEditor(editor_id) => {
                if editor_id < self.editors.len() {
                    self.active_editor = editor_id;
                }
            }
        }
    }

    /// Renders the entire editor interface
    fn view(&self) -> Element<Message> {
        let mut row = Row::new();

        for (index, editor) in self.editors.iter().enumerate() {
            let is_focused = index == self.active_editor;

            // NOTE: Is cloning the editor like this really a good idea?
            let configured_editor = editor.clone().focused(is_focused);
            row = row.push(configured_editor);
        }

        row.into()
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
