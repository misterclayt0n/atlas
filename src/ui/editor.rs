use iced::{
    advanced::{layout, renderer, Text, Widget},
    alignment, Border, Color, Element, Renderer, Shadow, Size, Theme,
};

use crate::Message;

/// Custom widget that handles the visual representation of text content.
/// Responsible for rendering text, cursor, and handling visual aspects.
pub struct Editor {
    content: String,
}

impl Editor {
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
        }
    }
}

pub fn create_editor(content: impl Into<String>) -> Editor {
    Editor::new(content)
}

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer> for Editor
where
    Renderer: renderer::Renderer + iced::advanced::text::Renderer, // This is used to render some text.
{
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

        // Draw background.
        renderer.fill_quad(
            renderer::Quad {
                bounds,
                border: Border {
                    color: Color::WHITE,
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
                content: self.content.clone(),
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
    }
}

// Helper function to create the widget
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
