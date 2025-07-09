use ropey::Rope;
use unicode_segmentation::UnicodeSegmentation;

use crate::{
    cursor::{MoveOpts, TextPosition},
    MultiCursor,
};

/// Represents a text buffer in the editor.
/// Handles the actual content storage and text manipulation operations.
#[derive(Debug, Clone, Default)]
pub struct Buffer {
    pub content: Rope,
    pub name: String,
    // TODO: Add file_path, modified.
}

/// Macro to handle multi-cursor operations with proper ordering.
///
/// It abstracts the common pattern of processing multiple cursors in a specific order to avoid
/// position invalidation when modifying the buffer.
///
/// - "Ascending": Process cursors from left to right (lowest offset to highest), which means they're used mostly for insertions.
/// - "Descending": Process cursors from right to left (highest offset to lowest), which means they're mostly used for deletions.
///
/// Usage:
/// ```
/// multi_cursor_operation(multi_cursor, ascending, idx => {
///     // Your operation code here using idx
/// }};
/// ```
macro_rules! multi_cursor_operation {
    ($multi_cursor:expr, ascending, $idx: ident => $body:block) => {{
        // Collect indices and sort by offset (ascending).
        let mut cursor_indices: Vec<usize> = (0..$multi_cursor.cursors.len()).collect();
        cursor_indices.sort_by_key(|&i| $multi_cursor.cursors[i].position().offset);

        // Process each cursor with the body you want.
        for $idx in cursor_indices {
            $body
        }
    }};

    ($multi_cursor:expr, descending, $idx: ident => $body:block) => {{
        // Collect indices and sort by offset (descending).
        let mut cursor_indices: Vec<usize> = (0..$multi_cursor.cursors.len()).collect();
        cursor_indices
            .sort_by_key(|&i| std::cmp::Reverse($multi_cursor.cursors[i].position().offset));

        // Process each cursor with the body you want.
        for $idx in cursor_indices {
            $body
        }
    }};
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
        assert!(
            line < self.content.len_lines(),
            "Line index out of range ({})",
            line
        );
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
        self.validate_offset(offset);
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
        self.validate_offset(offset);
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

    pub fn insert_char(&mut self, mc: &mut MultiCursor, c: char) {
        multi_cursor_operation!(mc, ascending, idx => {
            let pos = mc.cursors[idx].position();
            self.validate_position(&pos);

            // Insert character at current position.
            self.content.insert_char(pos.offset, c);

            // Move this cursor to the position after the inserted character.
            let new_pos = TextPosition::new(pos.line, pos.col + 1, pos.offset + 1);
            self.validate_position(&new_pos);
            mc.cursors[idx].move_to(new_pos, MoveOpts { anchor: None, update_preferred_col: true}, self);

            // Update positions of all other cursors affected by this insertion.
            self.update_cursors_after_modification(mc, pos.offset, 1, idx);
        });
    }

    pub fn insert_text(&mut self, mc: &mut MultiCursor, s: &str) {
        multi_cursor_operation!(mc, ascending, idx => {
            let pos = mc.cursors[idx].position();
            self.validate_position(&pos);

            // Insert text at current position.
            self.content.insert(pos.offset, s);
            let char_count = s.chars().count();

            // Calculate new position for this cursor.
            let new_pos = if s.contains('\n') {
                let new_offset = pos.offset + char_count;
                self.validate_offset(new_offset);
                let new_line = self.content.char_to_line(new_offset);
                let line_start = self.content.line_to_char(new_line);
                TextPosition::new(new_line, new_offset - line_start, new_offset)
            } else {
                TextPosition::new(
                    pos.line,
                    pos.col + s.graphemes(true).count(),
                    pos.offset + char_count,
                )
            };
            self.validate_position(&new_pos);
            mc.cursors[idx].move_to(new_pos, MoveOpts { anchor: None, update_preferred_col: true}, self);

            // Update positions of all other cursors affected by this insertion.
            self.update_cursors_after_modification(mc, pos.offset, char_count as isize, idx);
        });
    }

    pub fn backspace(&mut self, mc: &mut MultiCursor) {
        multi_cursor_operation!(mc, descending, idx => {
            let pos = mc.cursors[idx].position();

            if pos.offset == 0 {
                continue; // We can't backspace at the beginning of the buffer.
            }

            let start = self.prev_grapheme_offset(pos.offset);
            let deleted_len = pos.offset - start;

            // Actually perform the deletion.
            self.content.remove(start..pos.offset);

            // After deletion, the cursor should be at the start position.
            let new_offset = start;
            let new_line = self.content.char_to_line(new_offset);
            let line_start = self.content.line_to_char(new_line);
            let new_col = new_offset - line_start;
            let new_pos = TextPosition::new(new_line, new_col, new_offset);

            self.validate_position(&new_pos);
            mc.cursors[idx].move_to(new_pos, MoveOpts { anchor: None, update_preferred_col: true}, self);

            // Update positions of all other cursors affected by this deletion.
            self.update_cursors_after_modification(
                mc,
                start,
                -(deleted_len as isize),
                idx,
            );
        });

        // Ensure all positions are consistent.
        mc.refresh_positions(self);
    }

    pub fn delete(&mut self, mc: &mut MultiCursor) {
        multi_cursor_operation!(mc, descending, idx => {
            let pos = mc.cursors[idx].position();
            let end = self.next_grapheme_offset(pos.offset);
            let deleted_len = end - pos.offset; // Length of the deleted grapheme.

            // Perform the deletion.
            self.content.remove(pos.offset..end);

            // Update positions of all other cursors affected by this deletion.
            self.update_cursors_after_modification(
                mc,
                pos.offset,
                -(deleted_len as isize),
                idx,
            );
        });

        // Ensure all positions are consistent.
        mc.refresh_positions(self);
    }

    pub fn delete_selection(&mut self, mc: &mut MultiCursor) {
        // Right to left so later deletions don't invalidate earlier offsets.
        multi_cursor_operation!(mc, descending, idx => {
            let (start, end) = mc.cursors[idx].get_selection_range();
            
            self.validate_position(&start);
            self.validate_position(&end);

            let del_start = start.offset;
            let del_end   = self.next_grapheme_offset(end.offset);
            
            self.content.remove(del_start .. del_end);
            
            let mut new_pos = start;
            new_pos.offset  = del_start;
            new_pos.line    = self.content.char_to_line(del_start);
            let line_start  = self.content.line_to_char(new_pos.line);
            new_pos.col     = del_start - line_start;
            
            // Collapse selection at start.
            mc.cursors[idx].move_to(
                new_pos,
                MoveOpts { anchor: None, update_preferred_col: false },
                self,
            );

            // Shift the other cursors.
            self.update_cursors_after_modification(
                mc,
                del_start,
                -((del_end - del_start) as isize),
                idx
            );
        });

        mc.refresh_positions(self);
    }

    pub fn insert_newline(&mut self, multi_cursor: &mut crate::MultiCursor) {
        multi_cursor_operation!(multi_cursor, ascending, idx => {
            let pos = multi_cursor.cursors[idx].position();

            // Insert newline at current position.
            self.content.insert_char(pos.offset, '\n');

            // Move this cursor to the start of the new line.
            let new_line = pos.line + 1;
            let new_offset = self.content.line_to_char(new_line);
            let new_pos = TextPosition::new(new_line, 0, new_offset);
            self.validate_position(&new_pos);

            multi_cursor.cursors[idx].move_to(new_pos, MoveOpts { anchor: None, update_preferred_col: true}, self);

            // Update positions of all other cursors affected by this insertion.
            self.update_cursors_after_modification(multi_cursor, pos.offset, 1, idx);
        });
    }

    //
    // Correctness.
    //

    /// Helper function to update cursor positions after buffer modifications.
    /// This handles the common pattern of updating all cursors that come after a modification point.
    fn update_cursors_after_modification(
        &self,
        multi_cursor: &mut crate::MultiCursor,
        modification_offset: usize,
        offset_delta: isize, // Positive for insertions, negative for deletions.
        skip_cursor_idx: usize,
    ) {
        for (cursor_idx, cursor) in multi_cursor.cursors.iter_mut().enumerate() {
            if cursor_idx != skip_cursor_idx {
                let cursor_pos = cursor.position();
                let should_update = if offset_delta > 0 {
                    // For insertions, update cursors at or after the modification point.
                    cursor_pos.offset >= modification_offset
                } else {
                    // For deletions, update cursors after the modification point.
                    cursor_pos.offset > modification_offset
                };

                if should_update {
                    let new_offset = if offset_delta > 0 {
                        cursor_pos.offset + offset_delta as usize
                    } else {
                        cursor_pos.offset - (-offset_delta) as usize
                    };

                    let mut updated_pos = TextPosition {
                        offset: new_offset,
                        line: cursor_pos.line,
                        col: cursor_pos.col,
                    };

                    // Recalculate line and column based on new offset.
                    updated_pos.line = self.content.char_to_line(updated_pos.offset);
                    let line_start = self.content.line_to_char(updated_pos.line);
                    updated_pos.col = updated_pos.offset - line_start;

                    self.validate_position(&updated_pos);
                    cursor.move_to(
                        updated_pos,
                        MoveOpts {
                            anchor: None,
                            update_preferred_col: false,
                        },
                        self,
                    );
                }
            }
        }
    }

    pub fn validate_position(&self, pos: &TextPosition) {
        assert!(
            pos.line < self.content.len_lines(),
            "Line {} exceeds buffer lines {}",
            pos.line,
            self.content.len_lines()
        );
        assert!(
            pos.offset <= self.content.len_chars(),
            "Offset {} exceeds total characters {}",
            pos.offset,
            self.content.len_chars()
        );
        assert_eq!(
            pos.offset,
            self.grapheme_col_to_offset(pos.line, pos.col),
            "Offset {} does not match grapheme position for line {}, col {}",
            pos.offset,
            pos.line,
            pos.col
        );
    }

    pub fn validate_offset(&self, offset: usize) {
        assert!(
            offset <= self.content.len_chars(),
            "Offset {} exceeds total characters {}",
            offset,
            self.content.len_chars()
        );
    }
}
