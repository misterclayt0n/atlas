use atlas_engine::{Buffer, MultiCursor, Message, VimMode};
use atlas_vim::{execute, KeyEvent, VimEngine};
use iced::{
    advanced::{
        graphics::core::{event, widget},
        layout, mouse, renderer,
        text::Paragraph as _,
        widget::Tree,
        Clipboard, Layout, Shell, Text, Widget,
    },
    alignment,
    keyboard::{self, Key},
    widget::span,
    Border, Color, Element, Event, Point, Rectangle, Renderer, Shadow, Size, Theme,
};
use iced_graphics::{core::SmolStr, text::Paragraph};

/// Custom widget that handles the visual representation of text content.
/// Responsible for rendering text, cursor, and handling visual aspects.
#[derive(Clone)]
pub struct Editor {
    pub buffer: Buffer,
    pub multi_cursor: MultiCursor,
    scroll_offset: Point,
    vim: VimEngine,
}

#[derive(Debug)]
struct EditorState {
    is_focused: bool,
    // Cached values
    char_width: Option<f32>,
    line_height: Option<f32>,
    bounds: Rectangle,
}

impl Default for EditorState {
    fn default() -> Self {
        Self {
            is_focused: true, // This is for coding experience.
            char_width: None,
            line_height: None,
            bounds: Rectangle::default(),
        }
    }
}

impl Editor {
    const MARGIN_LINES: usize = 3;
    const MARGIN_COL: usize = 8;

    pub fn new() -> Self {
        Self {
            buffer: Buffer::new("", "Atlas"),
            multi_cursor: MultiCursor::default(),
            vim: VimEngine::default(),
            scroll_offset: Point::new(0.0, 0.0),
        }
    }

    fn char_width(&self, renderer: &impl iced::advanced::text::Renderer<Font = iced::Font>) -> f32 {
        // Create a paragraph with a single character to get precise width.
        // NOTE: We probably need to cache this.
        let bounds = Size::new(1000.0, 100.0);

        // We assume all characters have the same width, hence only monospaced fonts work.
        let paragraph = self.create_paragraph("M", bounds, renderer);

        if let Some(run) = paragraph.buffer().layout_runs().next() {
            if let Some(glyph) = run.glyphs.first() {
                return glyph.w;
            }
        }

        // Fallback.
        println!("Got to fallback");
        let size: f32 = renderer.default_size().into();
        size * 0.6
    }

    pub fn create_paragraph(
        &self,
        content: &str,
        bounds: iced::Size,
        renderer: &impl iced::advanced::text::Renderer<Font = iced::Font>,
    ) -> Paragraph {
        let font_size: f32 = renderer.default_size().into();

        iced_graphics::text::Paragraph::with_spans::<()>(iced::advanced::text::Text {
            bounds,
            content: &[span(content).to_static()],
            size: font_size.into(), // Use renderer font size.
            shaping: iced::advanced::text::Shaping::Basic,
            wrapping: iced::advanced::text::Wrapping::None,
            horizontal_alignment: iced::alignment::Horizontal::Left,
            vertical_alignment: iced::alignment::Vertical::Top,
            line_height: iced::widget::text::LineHeight::Relative(1.2),
            font: renderer.default_font(),
        })
    }

    fn line_height(&self, renderer: &impl iced::advanced::text::Renderer) -> f32 {
        let size: f32 = renderer.default_size().into();
        return size * 1.2;
    }

    fn ensure_cursor_visible(&mut self, bounds: Rectangle, char_width: f32, line_height: f32) {
        let cursor_pos = self.multi_cursor.position();
        let cursor_x = cursor_pos.col as f32 * char_width;
        let cursor_y = cursor_pos.line as f32 * line_height;

        // Defining vertical limits.
        let top_limit = self.scroll_offset.y + Self::MARGIN_LINES as f32 * line_height;
        let bottom_limit =
            self.scroll_offset.y + bounds.height - (Self::MARGIN_LINES + 1) as f32 * line_height;

        // Vertical scrolling
        if cursor_y < top_limit {
            self.scroll_offset.y = (cursor_y - Self::MARGIN_LINES as f32 * line_height).max(0.0);
        } else if cursor_y > bottom_limit {
            self.scroll_offset.y =
                (cursor_y + (Self::MARGIN_LINES + 1) as f32 * line_height) - bounds.height;
        }

        // Defining horizontal limits.
        let left_limit = self.scroll_offset.x + Self::MARGIN_COL as f32 * char_width;
        let right_limit =
            self.scroll_offset.x + bounds.width - (Self::MARGIN_COL + 1) as f32 * char_width;

        // Horizontal scrolling
        if cursor_x < left_limit {
            self.scroll_offset.x = (cursor_x - Self::MARGIN_COL as f32 * char_width).max(0.0);
        } else if cursor_x > right_limit {
            self.scroll_offset.x =
                (cursor_x + (Self::MARGIN_COL + 1) as f32 * char_width) - bounds.width;
        }
    }

    //
    // Drawing
    //

    /// Draws the cursor depending upon the current vim mode.
    fn draw_cursor(
        &self,
        renderer: &mut impl iced::advanced::text::Renderer,
        cursor: &atlas_engine::Cursor,
        position: Point,
        char_width: f32,
        line_height: f32,
        layout: iced::advanced::Layout<'_>,
    ) {
        let cursor_bounds = match self.vim.mode {
            VimMode::Normal | VimMode::Visual => Rectangle {
                x: position.x,
                y: position.y,
                width: char_width, // Block, basically.
                height: line_height,
            },
            VimMode::Insert => Rectangle {
                x: position.x,
                y: position.y,
                width: 2.0,
                height: line_height,
            },
        };

        // Get character under the cursor.
        let char_under_cursor =
            if let Some(character) = self.buffer.content.get_char(cursor.position().offset) {
                character
            } else {
                ' '
            };

        let cursor_background = match self.vim.mode {
            VimMode::Normal | VimMode::Visual => Color::WHITE,
            VimMode::Insert => Color::WHITE,
        };

        let text_color = match self.vim.mode {
            VimMode::Normal | VimMode::Visual => Color::BLACK,
            _ => Color::WHITE,
        };

        renderer.fill_quad(
            renderer::Quad {
                bounds: cursor_bounds,
                ..Default::default()
            },
            cursor_background,
        );

        // Draw character (for Normal/Visual modes) inside the cursor block.
        if self.vim.mode != VimMode::Insert {
            renderer.fill_text(
                Text {
                    content: char_under_cursor.to_string(),
                    bounds: cursor_bounds.size(),
                    size: renderer.default_size(),
                    line_height: line_height.into(),
                    font: renderer.default_font(),
                    horizontal_alignment: alignment::Horizontal::Center,
                    vertical_alignment: alignment::Vertical::Center,
                    shaping: iced::widget::text::Shaping::Basic,
                    wrapping: iced::widget::text::Wrapping::None,
                },
                cursor_bounds.center(),
                text_color,
                layout.bounds(),
            )
        }
    }

    /// Draws the visual selection background.
    fn draw_selection(
        &self,
        renderer: &mut impl iced::advanced::text::Renderer,
        bounds: Rectangle,
        char_width: f32,
        line_height: f32,
    ) {
        for cursor in self.multi_cursor.all_cursors() {
            if let Some((start, end)) = cursor.get_selection() {
                // Selection color.
                let selection_color = Color::from_rgba(0.3, 0.5, 0.8, 0.3);

                if start.line == end.line {
                    // Single line selection
                    let start_x = bounds.x + (start.col as f32 * char_width - self.scroll_offset.x);
                    let start_y = bounds.y + (start.line as f32 * line_height - self.scroll_offset.y);
                    let mut width = (end.col - start.col) as f32 * char_width;
                    
                    // Ensure minimum width for empty selections (like newlines)
                    if width < char_width * 0.5 {
                        width = char_width * 0.5;
                    }

                    let selection_bounds = Rectangle {
                        x: start_x,
                        y: start_y,
                        width,
                        height: line_height,
                    };

                    renderer.fill_quad(
                        renderer::Quad {
                            bounds: selection_bounds,
                            ..Default::default()
                        },
                        selection_color,
                    );
                } else {
                    // Multi-line selection
                    for line in start.line..=end.line {
                        let line_y = bounds.y + (line as f32 * line_height - self.scroll_offset.y);

                        let (start_col, end_col) = if line == start.line {
                            // First line: from start position to end of line
                            (start.col, self.buffer.grapheme_len(line))
                        } else if line == end.line {
                            // Last line: from beginning to end position
                            (0, end.col)
                        } else {
                            // Middle lines: entire line
                            (0, self.buffer.grapheme_len(line))
                        };

                        let start_x = bounds.x + (start_col as f32 * char_width - self.scroll_offset.x);
                        let mut width = (end_col - start_col) as f32 * char_width;
                        
                        // For empty lines or zero-width selections, show at least a small highlight
                        if width < char_width * 0.5 {
                            width = char_width * 0.5;
                        }

                        let selection_bounds = Rectangle {
                            x: start_x,
                            y: line_y,
                            width,
                            height: line_height,
                        };

                        renderer.fill_quad(
                            renderer::Quad {
                                bounds: selection_bounds,
                                ..Default::default()
                            },
                            selection_color,
                        );
                    }
                }
            }
        }
    }
}

impl<Theme, Renderer> Widget<Message, Theme, Renderer> for Editor
where
    Renderer: renderer::Renderer + iced::advanced::text::Renderer<Font = iced::Font>, // This is used to render some text.
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
        tree: &mut iced::advanced::widget::Tree,
        renderer: &Renderer,
        limits: &iced::advanced::layout::Limits,
    ) -> iced::advanced::layout::Node {
        let state = tree.state.downcast_mut::<EditorState>();

        if state.char_width.is_none() {
            state.char_width = Some(self.char_width(renderer));
            state.line_height = Some(self.line_height(renderer));
        }

        state.bounds = Rectangle {
            x: 0.0,
            y: 0.0,
            width: limits.max().width,
            height: limits.max().height,
        };

        layout::Node::new(limits.max())
    }

    fn draw(
        &self,
        tree: &iced::advanced::widget::Tree,
        renderer: &mut Renderer,
        _theme: &Theme,
        _style: &renderer::Style,
        layout: iced::advanced::Layout<'_>,
        _cursor: iced::advanced::mouse::Cursor,
        _viewport: &iced::Rectangle,
    ) {
        let bounds = layout.bounds();
        let state = tree.state.downcast_ref::<EditorState>();

        let char_w = state
            .char_width
            .unwrap_or_else(|| self.char_width(renderer));
        let line_height = state
            .line_height
            .unwrap_or_else(|| self.line_height(renderer));

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

        // Calculate visible line range.
        let first_line = (self.scroll_offset.y / line_height).floor() as usize;
        let visible_lines = (bounds.height / line_height).ceil() as usize;
        let total_lines = self.buffer.content.len_lines();
        let end_line = (first_line + visible_lines).min(total_lines);

        // Calculate visible column range.
        let first_col = (self.scroll_offset.x / char_w).floor() as usize;
        let visible_cols = (bounds.width / char_w).ceil() as usize;

        // Draw selection background.
        self.draw_selection(renderer, bounds, char_w, line_height);

        // Render each visible line.
        for line_idx in first_line..end_line {
            let visible_content = self
                .buffer
                .grapheme_substring(line_idx, first_col, visible_cols);
            let y = bounds.y + (line_idx as f32 * line_height - self.scroll_offset.y);
            let position = Point::new(bounds.x, y);

            renderer.fill_text(
                Text {
                    content: visible_content,
                    bounds: Size::new(bounds.width, line_height), // Size per line.
                    size: renderer.default_size(),
                    line_height: 1.2.into(),
                    font: renderer.default_font(),
                    horizontal_alignment: alignment::Horizontal::Left,
                    vertical_alignment: alignment::Vertical::Top,
                    shaping: iced::widget::text::Shaping::Basic,
                    wrapping: iced::widget::text::Wrapping::None,
                },
                position,
                iced::Color::WHITE,
                bounds, // Clip to widget bounds.
            );
        }

        // Draw all cursors.
        for cursor in self.multi_cursor.all_cursors() {
            let pos = cursor.position();
            let cursor_x = bounds.x + (pos.col as f32 * char_w - self.scroll_offset.x);
            let cursor_y = bounds.y + (pos.line as f32 * line_height - self.scroll_offset.y);
            self.draw_cursor(
                renderer,
                cursor,
                Point::new(cursor_x, cursor_y),
                char_w,
                line_height,
                layout,
            );
        }
    }

    fn on_event(
        &mut self,
        tree: &mut Tree,
        event: Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        _shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle,
    ) -> event::Status {
        // Access our custom state.
        let editor_state = tree.state.downcast_mut::<EditorState>();
        let char_width = editor_state
            .char_width
            .unwrap_or_else(|| self.char_width(renderer));
        let line_height = self.line_height(renderer);
        editor_state.bounds = layout.bounds();

        match event {
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
                mouse::Event::WheelScrolled { delta } => {
                    if cursor.is_over(layout.bounds()) {
                        match delta {
                            mouse::ScrollDelta::Lines { y, .. } => {
                                self.scroll_offset.y =
                                    (self.scroll_offset.y - y * line_height).max(0.0);
                            }
                            mouse::ScrollDelta::Pixels { y, .. } => {
                                self.scroll_offset.y = (self.scroll_offset.y - y).max(0.0);
                            }
                        }

                        return event::Status::Captured;
                    }
                }
                _ => {}
            },

            Event::Keyboard(keyboard::Event::KeyPressed { key, text, .. }) => {
                // Only capture if we are focused.
                if !editor_state.is_focused {
                    return event::Status::Ignored;
                }

                let maybe_action =
                    translate_to_keyevent(&key, &text).and_then(|ke| self.vim.handle_key(ke));

                if let Some(action) = maybe_action {
                    execute(action, &mut self.buffer, &mut self.multi_cursor, &self.vim.mode);
                    self.ensure_cursor_visible(editor_state.bounds, char_width, line_height);
                    return event::Status::Captured;
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

fn translate_to_keyevent(key: &Key, text: &Option<SmolStr>) -> Option<KeyEvent> {
    let text_str = text.as_ref().map(|t| t.to_string());

    match key.as_ref() {
        Key::Named(keyboard::key::Named::Escape) => Some(KeyEvent::Esc),
        Key::Named(keyboard::key::Named::Backspace) => Some(KeyEvent::Backspace),
        Key::Named(keyboard::key::Named::Enter) => Some(KeyEvent::Enter),
        Key::Character(s) => {
            if s.len() == 1 {
                // Use text if available, fallback to key.
                let c = text_str.unwrap_or(s.to_string());
                Some(KeyEvent::Key {
                    key: Key::Character(SmolStr::new(&c)),
                    text: Some(c),
                })
            } else {
                Some(KeyEvent::Key {
                    key: key.clone(),
                    text: text_str,
                })
            }
        }
        _ => Some(KeyEvent::Key {
            key: key.clone(),
            text: text_str,
        }),
    }
}
