use ropey::Rope;

/// Represents a text buffer in the editor.
/// Handles the actual content storage and text manipulation operations.
#[derive(Debug, Clone, Default)]
pub struct Buffer {
    pub content: Rope,
    pub name: String,
    // TODO: Add file_path, modified.
}

impl Buffer {
    pub fn new(content: &str, name: &str) -> Self {
        Self {
            content: Rope::from_str(content),
            name: name.to_string(),
        }
    }

    pub fn visible_line_content(&self, line: usize) -> String {
        let line_content = self.content.line(line);
        let content_str = line_content.to_string();

        content_str
            .trim_end_matches('\r')
            .trim_end_matches('\n')
            .to_string()
    }

    pub fn visual_line_length(&self, line: usize) -> usize {
        self.visible_line_content(line).chars().count()
    }

    pub fn insert_char(&mut self, offset: usize, c: char) {
        self.content.insert_char(offset, c)
    }

    pub fn remove_char(&mut self, offset: usize) -> Option<char> {
        if offset < self.content.len_chars() {
            let c = self.content.char(offset);
            self.content.remove(offset..offset + 1);
            Some(c)
        } else {
            None
        }
    }

    pub fn backspace(&mut self, offset: usize) -> Option<char> {
        if offset > 0 {
            let c = self.content.char(offset - 1);
            self.content.remove(offset - 1..offset);
            Some(c)
        } else {
            None
        }
    }
}
