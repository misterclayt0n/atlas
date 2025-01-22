use iced::{
    widget::{container, text, Column},
    Element, Length, Theme,
};

/// Main structure for manipulating text.
#[derive(Debug, Clone, Default)]
pub struct Buffer {
    pub content: String, // TODO: Refactor this to a Rope.
                         // TODO: Add line_count.
}

impl Buffer {
    pub fn new() -> Self {
        Self {
            content: String::new(),
        }
    }

    pub fn insert_char(&mut self, char: char, position: usize) {
        self.content.insert(position, char);
    }
}

#[derive(Default)]
pub struct Cursor {
    pub row: usize,
    pub col: usize,
    pub position: usize, // Linear position in the buffer.
}

impl Cursor {
    pub fn new() -> Self {
        Self {
            row: 0,
            col: 0,
            position: 0,
        }
    }
}

#[derive(Debug, Clone)]
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

pub struct Atlas {
    pub current_buffer: Buffer,
    pub cursor: Cursor,
    pub theme: Theme,
    pub fuck: bool,
}

impl Default for Atlas {
    fn default() -> Self {
        Self {
            current_buffer: Buffer::default(),
            cursor: Cursor::default(),
            theme: Theme::Dark,
            fuck: true
        }
    }
}

impl Atlas {
    fn title(&self) -> String {
        if self.fuck == true {
            return "Got fucked".to_string();
        }

        return "Not fucked at all".to_string();
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::TextInput(text) => self.current_buffer.content = text,
            Message::CursorMove(_) => {}
        }
    }

    fn view(&self) -> Element<Message> {
        // Create a column that will hold our widgets vertically.
        let content = Column::new()
            .push(text(self.title()))
            .push(text(&self.current_buffer.content))
            // Show buffer content.
            .width(Length::Fill)
            .spacing(20); // Add some spacing in between

        // Wrap it in a container.
        container(content).width(Length::Fill).height(Length::Fill).into()
    }
}

fn main() -> iced::Result {
    iced::application(Atlas::title, Atlas::update, Atlas::view).run()
}
