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
    engine::{buffer::Buffer, cursor::Cursor},
    CursorMovement, Message,
};

/// Custom widget that handles the visual representation of text content.
/// Responsible for rendering text, cursor, and handling visual aspects.
#[derive(Clone)]
pub struct Editor {
    pub buffer: Buffer,
    pub cursor: Cursor,
}

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
        return size * 0.6; // Approximation for monospace.
    }

    fn line_height(&self, renderer: &impl iced::advanced::text::Renderer) -> f32 {
        let size: f32 = renderer.default_size().into();
        return size * 1.2;
    }

    pub fn new() -> Self {
        Self {
            buffer: Buffer::new("Amazing", "Yes"),
            cursor: Cursor::new(),
        }
    }

    pub fn move_cursor(&mut self, movement: CursorMovement) {
        let new_position = match movement {
            CursorMovement::Left => self.cursor.move_left(&self.buffer),
            CursorMovement::Right => self.cursor.move_right(&self.buffer),
            CursorMovement::Up => self.cursor.move_up(&self.buffer),
            CursorMovement::Down => self.cursor.move_down(&self.buffer),
        };

        if let Some(position) = new_position {
            match &mut self.cursor {
                Cursor::Normal {
                    position: pos,
                    preferred_column,
                } => {
                    *pos = position;
                    // Update preferred column for vertical movements.
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
    fn tag(&self) -> widget::tree::Tag {
        widget::tree::Tag::of::<EditorState>()
    }

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
        // Create a simple layout node that fills the available space.
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
        // Access our custom state.
        let editor_state = tree.state.downcast_mut::<EditorState>();

        match event {
            // 1) Mouse: handle focus/unfocus.
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

            // 2) Keyboard input.
            Event::Keyboard(keyboard::Event::KeyPressed { key, text, .. }) => {
                // Only capture if we are focused.
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
                        Key::Named(keyboard::key::Named::Enter) => {
                            shell.publish(Message::InsertChar('\n'));
                            return event::Status::Captured;
                        }
                        Key::Named(keyboard::key::Named::Tab) => {
                            shell.publish(Message::InsertChar('\t'));
                            return event::Status::Captured;
                        }
                        Key::Named(keyboard::key::Named::Space) => {
                            // Handle dead keys here.
                            // NOTE: This is the easiest way I've found to handle dead keys
                            // There could exist some more elegant solution to this particular problem, but I'm too
                            // lazy to find it, so we'll leave it like this for now.
                            if let Some(dead_key) = text {
                                for c in dead_key.chars() {
                                    if !c.is_control() {
                                        shell.publish(Message::InsertChar(c))
                                    }
                                }

                                return event::Status::Captured;
                            }

                            shell.publish(Message::InsertChar(' '));
                            return event::Status::Captured;
                        }
                        Key::Named(keyboard::key::Named::Backspace) => {
                            shell.publish(Message::Backspace);
                            return event::Status::Captured;
                        }
                        Key::Named(keyboard::key::Named::Delete) => {
                            shell.publish(Message::Delete);
                            return event::Status::Captured;
                        }

                        // Insert characters.
                        Key::Character(_) => {
                            if let Some(composed) = text {
                                // Insert each character from the final string
                                for c in composed.chars() {
                                    if !c.is_control() {
                                        shell.publish(Message::InsertChar(c));
                                    }
                                }
                                return event::Status::Captured;
                            }
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
