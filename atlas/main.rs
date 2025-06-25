use atlas_engine::Message;
use atlas_widgets::editor::Editor;
use iced::widget::pane_grid;
use iced::{
    Element,
    widget::pane_grid::{Axis, Pane},
};

/// Main application structure.
/// Manages the overall editor state and handles high-level operations.
pub struct Atlas {
    panes: pane_grid::State<Editor>,
    active_pane: Pane,
}

impl Default for Atlas {
    fn default() -> Self {
        let (panes, first_editor) = pane_grid::State::new(Editor::new());

        Self {
            panes,
            active_pane: first_editor,
        }
    }
}

impl Atlas {
    /// Generates the window title based on the active buffer.
    fn title(&self) -> String {
        "Atlas".into()
    }

    /// Handles all editor actions and updates state accordingly.
    fn update(&mut self, message: Message) {
        match message {
            Message::SplitVertical => {
                self.panes
                    .split(Axis::Vertical, self.active_pane, Editor::new());
            }
            Message::SplitHorizontal => {
                self.panes
                    .split(Axis::Horizontal, self.active_pane, Editor::new());
            }
            Message::Quit => {
                std::process::exit(0);
            }
            Message::PaneClicked(pane) => self.active_pane = pane,
            Message::Dragged(_) => {
                println!("do we even care about this one?");
            }
            Message::Resized(resize_event) => {
                self.panes.resize(resize_event.split, resize_event.ratio);
            }
            Message::CloseSplit => {
                if let Some((_removed_editor, sibling)) = self.panes.close(self.active_pane) {
                    self.active_pane = sibling;
                } else {
                    println!("no split to close");
                }
            }
        }
    }

    /// Renders the entire editor interface.
    fn view(&self) -> Element<Message> {
        pane_grid(&self.panes, |pane_id, editor, _| {
            let elem: Element<_> = editor.clone().focused(pane_id == self.active_pane).into();
            pane_grid::Content::new(elem)
        })
        .on_click(Message::PaneClicked)
        .on_drag(Message::Dragged)
        .on_resize(6, Message::Resized)
        .into()
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
