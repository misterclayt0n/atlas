use iced::{
    advanced::{
        graphics::core::{event, widget},
        layout, mouse, renderer,
        widget::Tree,
        Clipboard, Layout, Shell, Text, Widget,
    },
    alignment,
    keyboard::{self, Key},
    Border, Color, Element, Event, Rectangle, Renderer, Shadow, Size, Theme,
};

use crate::{
    engine::{
        buffer::Buffer,
        cursor::{Cursor, TextPosition},
    },
    CursorMovement, Message,
};

/// Custom widget that handles the visual representation of text content.
/// Responsible for rendering text, cursor, and handling visual aspects.
#[derive(Clone)]
pub struct Editor {
    buffer: Buffer,
    cursor: Cursor,
}

// Add a small focus flag to your Editor's state
#[derive(Debug)]
struct EditorState {
    is_focused: bool,
}

impl Default for EditorState {
    fn default() -> Self {
        Self { is_focused: true } // This is for coding experience.
    }
}

impl Editor {
    fn char_width(&self, renderer: &impl iced::advanced::text::Renderer) -> f32 {
        let size: f32 = renderer.default_size().into();
        return size; // Approximation for monospace
    }

    fn line_height(&self, renderer: &impl iced::advanced::text::Renderer) -> f32 {
        let size: f32 = renderer.default_size().into();
        return size * 1.2;
    }

    pub fn new(buffer: Buffer) -> Self {
        Self {
            buffer,
            cursor: Cursor::new(),
        }
    }

    // Movement calculations
    fn compute_position_left(&self) -> Option<TextPosition> {
        let current = self.cursor.position();

        if current.col > 0 {
            // Move left in the current line.
            Some(TextPosition::new(
                current.line,
                current.col - 1,
                current.offset - 1,
            ))
        } else if current.line > 0 {
            // Move to end of previous line
            let prev_line = self.buffer.content.line(current.line - 1);
            Some(TextPosition::new(
                current.line - 1,
                prev_line.len_chars(),
                current.offset - 1,
            ))
        } else {
            None
        }
    }

    fn compute_position_right(&self) -> Option<TextPosition> {
        let current = self.cursor.position();
        let visual_len = self.buffer.visual_line_length(current.line);

        println!("Current col: {}; Visual len: {}", current.col, visual_len);

        if current.col < visual_len {
            // Move right within the current line.
            Some(TextPosition::new(
                current.line,
                current.col + 1,
                current.offset + 1,
            ))
        } else if current.line < self.buffer.content.len_lines() - 1 {
            // Move to the start of next line.
            // TODO: Remove this behavior.
            Some(TextPosition::new(
                current.line + 1,
                0,
                self.buffer.content.line_to_byte(current.line + 1),
            ))
        } else {
            None
        }
    }

    fn compute_position_up(&self) -> Option<TextPosition> {
        let current = self.cursor.position();
        if current.line > 0 {
            let target_col = if let Some(preferred) = self.get_preferred_column() {
                preferred
            } else {
                current.col
            };

            let prev_line_len = self.buffer.content.line(current.line - 1).len_chars();
            let new_col = target_col.min(prev_line_len);

            Some(TextPosition::new(
                current.line - 1,
                new_col,
                self.calculate_offset(current.line - 1, new_col),
            ))
        } else {
            None
        }
    }

    fn compute_position_down(&self) -> Option<TextPosition> {
        let current = self.cursor.position();
        if current.line < self.buffer.content.len_lines() - 1 {
            let target_col = if let Some(preferred) = self.get_preferred_column() {
                preferred
            } else {
                current.col
            };

            let next_line_len = self.buffer.content.line(current.line + 1).len_chars();
            let new_col = target_col.min(next_line_len);

            Some(TextPosition::new(
                current.line + 1,
                new_col,
                self.buffer.content.line_to_byte(current.line + 1) + new_col,
            ))
        } else {
            None
        }
    }

    fn calculate_offset(&self, line: usize, col: usize) -> usize {
        self.buffer.content.line_to_byte(line) + col
    }

    fn get_preferred_column(&self) -> Option<usize> {
        match &self.cursor {
            Cursor::Normal {
                preferred_column, ..
            } => *preferred_column,
            _ => None,
        }
    }

    pub fn move_cursor(&mut self, movement: CursorMovement) {
        let new_position = match movement {
            CursorMovement::Left => self.compute_position_left(),
            CursorMovement::Right => self.compute_position_right(),
            CursorMovement::Up => self.compute_position_up(),
            CursorMovement::Down => self.compute_position_down(),
        };

        if let Some(position) = new_position {
            match &mut self.cursor {
                Cursor::Normal {
                    position: pos,
                    preferred_column,
                } => {
                    *pos = position;
                    // Update preferred column for vertical movements
                    if matches!(movement, CursorMovement::Up | CursorMovement::Down) {
                        *preferred_column = Some(position.col);
                    }
                }
                Cursor::Selection { active, .. } => {
                    *active = position;
                }
            }
        }
    }
}

impl<Theme, Renderer> Widget<Message, Theme, Renderer> for Editor
where
    Renderer: renderer::Renderer + iced::advanced::text::Renderer, // This is used to render some text.
    Message:,
{
    // 1) Tag your custom editor's state type
    fn tag(&self) -> widget::tree::Tag {
        widget::tree::Tag::of::<EditorState>()
    }

    // 2) Initialize the `EditorState` once, when your widget is first created
    fn state(&self) -> widget::tree::State {
        widget::tree::State::new(EditorState::default())
    }

    fn size(&self) -> iced::Size<iced::Length> {
        Size::new(iced::Length::Fill, iced::Length::Fill)
    }

    fn layout(
        &self,
        _tree: &mut iced::advanced::widget::Tree,
        _renderer: &Renderer,
        limits: &iced::advanced::layout::Limits,
    ) -> iced::advanced::layout::Node {
        // Create a simple layout node that fills the available space
        let size = limits.max();
        layout::Node::new(size)
    }

    fn draw(
        &self,
        _tree: &iced::advanced::widget::Tree,
        renderer: &mut Renderer,
        _theme: &Theme,
        _style: &renderer::Style,
        layout: iced::advanced::Layout<'_>,
        _cursor: iced::advanced::mouse::Cursor,
        _viewport: &iced::Rectangle,
    ) {
        let bounds = layout.bounds();
        let content = self.buffer.content.to_string();

        // Draw background.
        renderer.fill_quad(
            renderer::Quad {
                bounds,
                border: Border {
                    color: Color::BLACK,
                    width: 1.0,
                    radius: 0.0.into(),
                },
                shadow: Shadow::default(),
            },
            Color::from_rgb(0.1, 0.1, 0.1),
        );

        // Draw the text
        renderer.fill_text(
            Text {
                content,
                bounds: bounds.size(),
                size: renderer.default_size(),
                line_height: 1.2.into(),
                font: renderer.default_font(),
                horizontal_alignment: alignment::Horizontal::Left,
                vertical_alignment: alignment::Vertical::Top,
                shaping: iced::widget::text::Shaping::Basic,
                wrapping: iced::widget::text::Wrapping::None,
            },
            bounds.position(),
            iced::Color::WHITE, // Text color
            bounds,
        );

        // Draw cursor.
        let cursor_point = self
            .cursor
            .to_point(self.char_width(renderer), self.line_height(renderer));

        let cursor_bounds = Rectangle {
            x: bounds.x + cursor_point.x,
            y: bounds.y + cursor_point.y,
            width: 2.0,
            height: self.line_height(renderer),
        };

        renderer.fill_quad(
            renderer::Quad {
                bounds: cursor_bounds,
                border: Border::default(),
                shadow: Shadow::default(),
            },
            Color::WHITE,
        )
    }

    fn on_event(
        &mut self,
        tree: &mut Tree,
        event: Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle,
    ) -> event::Status {
        // Access our custom state
        let editor_state = tree.state.downcast_mut::<EditorState>();

        match event {
            // 1) Mouse: handle focus/unfocus
            Event::Mouse(mouse_event) => match mouse_event {
                mouse::Event::ButtonPressed(mouse::Button::Left) => {
                    // If clicked inside our widget, focus. Otherwise, unfocus.
                    if cursor.is_over(layout.bounds()) {
                        editor_state.is_focused = true;
                        return event::Status::Captured;
                    } else {
                        editor_state.is_focused = false;
                    }
                }
                _ => {}
            },

            // 2) Keyboard input
            Event::Keyboard(keyboard::Event::KeyPressed { key, .. }) => {
                // Only capture if we are focused
                if editor_state.is_focused {
                    match key {
                        Key::Named(keyboard::key::Named::ArrowUp) => {
                            shell.publish(Message::CursorMove(CursorMovement::Up));
                            return event::Status::Captured;
                        }
                        Key::Named(keyboard::key::Named::ArrowDown) => {
                            shell.publish(Message::CursorMove(CursorMovement::Down));
                            return event::Status::Captured;
                        }
                        Key::Named(keyboard::key::Named::ArrowLeft) => {
                            shell.publish(Message::CursorMove(CursorMovement::Left));
                            return event::Status::Captured;
                        }
                        Key::Named(keyboard::key::Named::ArrowRight) => {
                            shell.publish(Message::CursorMove(CursorMovement::Right));
                            return event::Status::Captured;
                        }

                        // TODO: Handle more keys.
                        _ => {}
                    }
                }
            }
            _ => {}
        }

        // If we did not capture anything:
        event::Status::Ignored
    }
}

// Helper function to create the widget.
impl<'a> Into<Element<'a, Message, Theme, Renderer>> for Editor
where
    Message: 'a,
    Theme: 'a + Default,
    Renderer: 'a + renderer::Renderer,
{
    fn into(self) -> Element<'a, Message, Theme, Renderer> {
        Element::new(self)
    }
}
