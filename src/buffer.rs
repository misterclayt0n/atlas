use ropey::Rope;
use unicode_segmentation::UnicodeSegmentation;

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
        assert!(
            line < self.content.len_lines(),
            "Line index out of range ({line})"
        );

        self.content
            .line(line)
            .to_string()
            .trim_end_matches(['\r', '\n'])
            .to_string()
    }

    pub fn grapheme_substring(&self, line: usize, start: usize, len: usize) -> String {
        let content = self.visible_line_content(line);
        content
            .graphemes(true)
            .skip(start)
            .take(len)
            .collect::<Vec<_>>()
            .join("")
    }

    pub fn visual_line_length(&self, line: usize) -> usize {
        self.visible_line_content(line).chars().count()
    }

    /// Number of grapheme clusters in the visible part of 'line'.
    pub fn grapheme_len(&self, line: usize) -> usize {
        self.visible_line_content(line).graphemes(true).count()
    }

    /// Translate (line, grapheme column) to absolute char offset.
    /// Used by the cursor when it needs the real Rope effect.
    pub fn grapheme_col_to_offset(&self, line: usize, col: usize) -> usize {
        assert!(
            line < self.content.len_lines(),
            "Line index out of range ({line})"
        );

        assert!(
            col <= self.grapheme_len(line),
            "Column {col} exceeds grapheme_len(line)"
        );

        let mut chars = 0;

        for (i, g) in self.visible_line_content(line).graphemes(true).enumerate() {
            if i == col {
                break;
            }

            chars += g.chars().count();
        }

        self.content.line_to_char(line) + chars
    }

    /// Given a char offset, return the previous grapheme boundary.
    pub fn prev_grapheme_offset(&self, offset: usize) -> usize {
        assert!(
            offset <= self.content.len_chars(),
            "Offset {offset} exceeds content length"
        );

        if offset == 0 {
            return 0;
        }

        // REFACTOR: Avoid using to_string(), too many allocations here.
        let slice = self.content.slice(..offset).to_string(); // Small: Only <= line.
        let mut last = 0;
        for (b, _) in slice.grapheme_indices(true) {
            last = b;
        }

        self.content.byte_to_char(last)
    }

    /// Next boundary.
    pub fn next_grapheme_offset(&self, offset: usize) -> usize {
        assert!(
            offset <= self.content.len_chars(),
            "Offset {offset} exceeds content length"
        );

        let total = self.content.len_chars();
        if offset >= total {
            return total;
        }

        let start_byte = self.content.char_to_byte(offset);
        let slice = self.content.slice(offset..).to_string();

        let next_byte_off_in_slice = slice
            .grapheme_indices(true)
            .nth(1)
            .map(|(b, _)| b)
            .unwrap_or(slice.len());

        self.content
            .byte_to_char(start_byte + next_byte_off_in_slice)
    }

    pub fn insert_char(&mut self, offset: usize, c: char) {
        assert!(
            offset <= self.content.len_chars(),
            "Insert out of bounds: {} > {}",
            offset,
            self.content.len_chars()
        );

        self.content.insert_char(offset, c)
    }

    pub fn delete(&mut self, offset: usize) {
        assert!(
            offset <= self.content.len_chars(),
            "Insert out of bounds: {} > {}",
            offset,
            self.content.len_chars()
        );

        let end = self.next_grapheme_offset(offset);
        self.content.remove(offset..end);
    }

    pub fn backspace(&mut self, offset: usize) {
        assert!(
            offset <= self.content.len_chars(),
            "Insert out of bounds: {} > {}",
            offset,
            self.content.len_chars()
        );

        if offset == 0 {
            return;
        }

        let start = self.prev_grapheme_offset(offset);
        self.content.remove(start..offset);
    }
}
