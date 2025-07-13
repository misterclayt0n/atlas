use iced::Point;

use super::buffer::Buffer;
use crate::EditorMode;

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct Cursor {
    anchor: TextPosition,            // Where the selection starts.
    active: TextPosition,            // Where it is currently.
    preferred_column: Option<usize>, // Global preferred column for all modes.
}

/// Represents a position in the text buffer.
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
pub struct TextPosition {
    pub line: usize,
    pub col: usize,
    pub offset: usize, // Linear position in the buffer (character count from start).
}

#[derive(Debug, PartialEq)]
enum CharClass {
    Whitespace,
    Word,
    Punctuation,
}

/// How the cursor move should behave.
#[derive(Default, Clone, Copy)]
pub struct MoveOpts {
    /// If `Some(anchor)` we keep / start a selection from `anchor` to `dest`.
    /// If `None` the selection is collapsed at `dest`.
    pub anchor: Option<TextPosition>,
    pub update_preferred_col: bool,
}

fn get_char_class(c: char, big_word: bool) -> CharClass {
    if c.is_whitespace() {
        CharClass::Whitespace
    } else if big_word || c.is_alphanumeric() || c == '_' {
        CharClass::Word
    } else {
        CharClass::Punctuation
    }
}

impl Cursor {
    pub fn new() -> Self {
        let pos = TextPosition::default();

        Self {
            preferred_column: None,
            anchor: pos,
            active: pos,
        }
    }

    pub fn position(&self) -> TextPosition {
        self.active
    }

    /// Converts cursor position to screen coordinates.
    pub fn _to_point(&self, char_width: f32, line_height: f32) -> Point {
        assert!(
            char_width > 0.0,
            "Character width {} must be positive",
            char_width
        );
        assert!(
            line_height > 0.0,
            "Line height {} must be positive",
            line_height
        );

        let pos = self.position();
        Point::new(pos.col as f32 * char_width, pos.line as f32 * line_height)
    }

    pub fn collapse_selection(&mut self) {
        self.anchor = self.active;
    }

    /// A selection exists if the anchor and active positions are different.
    pub fn has_selection(&self) -> bool {
        self.anchor != self.active
    }

    /// Get selection range if in selection mode.
    pub fn get_selection_range(&self) -> (TextPosition, TextPosition) {
        if self.anchor.offset <= self.active.offset {
            (self.anchor, self.active)
        } else {
            (self.active, self.anchor)
        }
    }

    //
    // Movement
    //

    pub fn move_left(&mut self, buffer: &Buffer, editor_mode: &EditorMode) -> Option<TextPosition> {
        let cur = self.position();
        buffer.validate_position(&cur);

        if cur.col == 0 {
            return None;
        }

        let new_col = cur.col - 1;
        let new_off = buffer.grapheme_col_to_offset(cur.line, new_col);
        let new_pos = TextPosition::new(cur.line, new_col, new_off);

        buffer.validate_position(&new_pos);

        let keep_anchor = matches!(editor_mode, EditorMode::Visual);
        self.move_to(
            new_pos,
            MoveOpts {
                anchor: if keep_anchor { Some(self.anchor) } else { None },
                update_preferred_col: true,
            },
            buffer,
        );

        Some(new_pos)
    }

    pub fn move_right(
        &mut self,
        buffer: &Buffer,
        editor_mode: &EditorMode,
    ) -> Option<TextPosition> {
        let cur = self.position();
        buffer.validate_position(&cur);

        // In Normal mode, cursor can't go past the last character
        // In Insert mode, cursor can go one position past the last character
        let max_col = self.get_max_col(editor_mode, buffer, cur.line);

        if cur.col >= max_col {
            return None;
        }

        let new_col = cur.col + 1;
        let new_off = buffer.grapheme_col_to_offset(cur.line, new_col);
        let new_pos = TextPosition::new(cur.line, new_col, new_off);

        buffer.validate_position(&new_pos);
        let keep_anchor = matches!(editor_mode, EditorMode::Visual);
        self.move_to(
            new_pos,
            MoveOpts {
                anchor: if keep_anchor { Some(self.anchor) } else { None },
                update_preferred_col: true,
            },
            buffer,
        );

        Some(new_pos)
    }

    pub fn move_up(&mut self, buffer: &Buffer, editor_mode: &EditorMode) -> Option<TextPosition> {
        let cur = self.position();
        buffer.validate_position(&cur);

        if cur.line == 0 {
            return None;
        }

        let target_line = cur.line - 1;
        let target_col = self.preferred_column.unwrap_or(cur.col);

        let max_col = self.get_max_col(editor_mode, buffer, target_line);
        let new_col = target_col.min(max_col);
        let new_off = buffer.grapheme_col_to_offset(target_line, new_col);
        let new_pos = TextPosition::new(cur.line - 1, new_col, new_off);

        buffer.validate_position(&new_pos);
        let keep_anchor = matches!(editor_mode, EditorMode::Visual);
        self.move_to(
            new_pos,
            MoveOpts {
                anchor: if keep_anchor { Some(self.anchor) } else { None },
                update_preferred_col: false,
            },
            buffer,
        );

        Some(new_pos)
    }

    pub fn move_down(&mut self, buffer: &Buffer, editor_mode: &EditorMode) -> Option<TextPosition> {
        let cur = self.position();
        buffer.validate_position(&cur);

        if cur.line + 1 >= buffer.content.len_lines() {
            return None;
        }

        let target_line = cur.line + 1;
        let target_col = self.preferred_column.unwrap_or(cur.col);

        let max_col = self.get_max_col(editor_mode, buffer, target_line);
        let new_col = target_col.min(max_col);
        let new_off = buffer.grapheme_col_to_offset(target_line, new_col);
        let new_pos = TextPosition::new(cur.line + 1, new_col, new_off);

        buffer.validate_position(&new_pos);
        let keep_anchor = matches!(editor_mode, EditorMode::Visual);
        self.move_to(
            new_pos,
            MoveOpts {
                anchor: if keep_anchor { Some(self.anchor) } else { None },
                update_preferred_col: false,
            },
            buffer,
        );

        Some(new_pos)
    }

    pub fn move_word_forward(
        &mut self,
        buffer: &Buffer,
        big_word: bool,
        editor_mode: &EditorMode,
    ) -> Option<TextPosition> {
        let total = buffer.content.len_chars();
        let start = self.position();
        buffer.validate_position(&start);

        let mut off = start.offset;
        if off >= total {
            return None;
        }

        let cur_class = get_char_class(buffer.content.char(off), big_word);
        while off < total && get_char_class(buffer.content.char(off), big_word) == cur_class {
            off += 1;
        }
        if off >= total {
            return None;
        }
        
        while off < total && buffer.content.char(off).is_whitespace() {
            let ch = buffer.content.char(off);
            if ch == '\n' {
                off += 1;
                if off < total {
                    let next_ch = buffer.content.char(off);
                    if next_ch == '\t' || next_ch == ' ' {
                        let line = buffer.content.char_to_line(off);
                        let line_start = buffer.content.line_to_char(line);
                        if off == line_start {
                            break; 
                        }
                    }
                }
            } else {
                off += 1;
            }
        }
        
        if off >= total {
            return None;
        }
        
        let landed_class = if off < total { get_char_class(buffer.content.char(off), big_word) } else { CharClass::Whitespace };
        
        match landed_class {
            CharClass::Word => {
                while off + 1 < total 
                    && buffer.content.char(off + 1) != '\n'
                    && get_char_class(buffer.content.char(off + 1), big_word) == CharClass::Word 
                {
                    off += 1;
                }
                
                if cur_class == CharClass::Punctuation && off + 1 < total && buffer.content.char(off + 1) != '\n' {
                    let next_class = get_char_class(buffer.content.char(off + 1), big_word);
                    if next_class == CharClass::Whitespace {
                        off += 1;
                    }
                }
            }
            CharClass::Punctuation => {
                while off + 1 < total 
                    && buffer.content.char(off + 1) != '\n'
                    && get_char_class(buffer.content.char(off + 1), big_word) == CharClass::Punctuation 
                {
                    off += 1;
                }
                
                if off + 1 < total && buffer.content.char(off + 1) != '\n' {
                    let next_class = get_char_class(buffer.content.char(off + 1), big_word);
                    if next_class == CharClass::Whitespace {
                        off += 1;
                    }
                }
            }
            CharClass::Whitespace => {
                // Already at whitespace, no need to move.
            }
        }

        if off < total && buffer.content.char(off) == '\n' && off + 1 < total {
            off += 1;
        }

        if big_word {
            while off < total
                && buffer.content.char(off).is_whitespace()
                && buffer.content.char(off) != '\n'
            {
                off += 1;
            }
        }

        if off >= total {
            return None;
        }

        let line = buffer.content.char_to_line(off);
        let col = off - buffer.content.line_to_char(line);
        let dest = TextPosition::new(line, col, off);
        buffer.validate_position(&dest);

        let start_class = get_char_class(buffer.content.char(start.offset), big_word);
        let end_class   = get_char_class(buffer.content.char(off), big_word);
        
        let keep_anchor = matches!(editor_mode, EditorMode::Visual) && start_class == end_class;
        
        self.move_to(
            dest,
            MoveOpts {
                anchor: if keep_anchor {
                    Some(self.anchor)
                } else {
                    Some(start)
                },
                update_preferred_col: true,
            },
            buffer,
        )
    }

    pub fn move_word_backward(
        &mut self,
        buffer: &Buffer,
        big_word: bool,
        editor_mode: &EditorMode,
    ) -> Option<TextPosition> {
        let start = self.position();
        buffer.validate_position(&start);

        if start.offset == 0 {
            return None;
        }

        let mut off = start.offset;
        
        let at_word_start = if start.offset == 0 {
            true
        } else {
            let cur_char = buffer.content.char(start.offset);
            let cur_class = get_char_class(cur_char, big_word);
            
            let prev_char = buffer.content.char(start.offset - 1);
            let prev_class = get_char_class(prev_char, big_word);
            
            prev_char.is_whitespace() || prev_char == '\n' || 
            (!big_word && cur_class == CharClass::Punctuation) ||
            (cur_class != prev_class)
        };
        
        if !at_word_start {
            let cur_char = buffer.content.char(off);
            let cur_class = get_char_class(cur_char, big_word);
            
            if !big_word && cur_class == CharClass::Punctuation {
            } else {
                while off > 0 {
                    let prev_char = buffer.content.char(off - 1);
                    if prev_char.is_whitespace() || prev_char == '\n' {
                        break;
                    }
                    let prev_class = get_char_class(prev_char, big_word);
                    if prev_class != cur_class {
                        break;
                    }
                    off -= 1;
                }
            }
        } else {
            let at_line_start = start.col == 0;
            
            off -= 1;
            
            let mut landed_char = buffer.content.char(off);
            let mut landed_class = get_char_class(landed_char, big_word);
            
            if landed_char == '\n' && off > 0 {
                off -= 1;

                landed_char = buffer.content.char(off);
                landed_class = get_char_class(landed_char, big_word);
            }
            
            let char_class = landed_class;
            
            if char_class == CharClass::Whitespace {
                let whitespace_start = off;
                
                while off > 0 {
                    let prev_char = buffer.content.char(off - 1);
                    if !prev_char.is_whitespace() || prev_char == '\n' {
                        break;
                    }
                    off -= 1;
                }
                
                let whitespace_len = whitespace_start - off + 1;
                let at_line_beginning = {
                    let line = buffer.content.char_to_line(off);
                    let line_start = buffer.content.line_to_char(line);
                    off == line_start
                };
                
                if whitespace_len == 1 && !at_line_beginning && off > 0 {
                    off -= 1;
                    
                    let new_char = buffer.content.char(off);
                    let new_class = get_char_class(new_char, big_word);
                    
                    if new_class == CharClass::Word {
                        while off > 0 {
                            let prev_char = buffer.content.char(off - 1);
                            if prev_char.is_whitespace() || prev_char == '\n' {
                                break;
                            }
                            let prev_class = get_char_class(prev_char, big_word);
                            if prev_class != new_class {
                                break;
                            }
                            off -= 1;
                        }
                    } else if new_class == CharClass::Punctuation && !big_word {
                        while off > 0 {
                            let prev_char = buffer.content.char(off - 1);
                            if prev_char.is_whitespace() || prev_char == '\n' {
                                break;
                            }
                            let prev_class = get_char_class(prev_char, big_word);
                            if prev_class != CharClass::Punctuation {
                                break;
                            }
                            off -= 1;
                        }
                    }
                } else {
                }
            } else if char_class == CharClass::Punctuation {
                if at_line_start && !big_word {
                    // NOTE: When at the beginning of a line, scan backwards through punctuation
                    // to find a meaningful boundary (like "()").
                    let mut scan_off = off;
                    while scan_off > 0 {
                        let ch = buffer.content.char(scan_off);
                        if ch.is_whitespace() || ch == '\n' {
                            break;
                        }
                        
                        let ch_class = get_char_class(ch, big_word);
                        if ch_class != CharClass::Punctuation {
                            break;
                        }
                        
                        if ch == '"' || ch == '\'' || ch == '(' || ch == '[' || ch == '{' || ch == '<' {
                            off = scan_off;
                            break;
                        }
                        
                        scan_off -= 1;
                    }
                } else if !big_word {
                    while off > 0 {
                        let prev_char = buffer.content.char(off - 1);
                        if prev_char.is_whitespace() || prev_char == '\n' {
                            break;
                        }
                        let prev_class = get_char_class(prev_char, big_word);
                        if prev_class != CharClass::Punctuation {
                            break;
                        }
                        off -= 1;
                    }
                } else {
                    while off > 0 {
                        let prev_char = buffer.content.char(off - 1);
                        if prev_char.is_whitespace() || prev_char == '\n' {
                            break;
                        }
                        let prev_class = get_char_class(prev_char, big_word);
                        if prev_class != char_class {
                            break;
                        }
                        off -= 1;
                    }
                }
            } else {
                while off > 0 {
                    let prev_char = buffer.content.char(off - 1);
                    if prev_char.is_whitespace() || prev_char == '\n' {
                        break;
                    }
                    let prev_class = get_char_class(prev_char, big_word);
                    if prev_class != char_class {
                        break;
                    }
                    off -= 1;
                }
            }
        }
        
        let line = buffer.content.char_to_line(off);
        let col = off - buffer.content.line_to_char(line);
        let dest = TextPosition::new(line, col, off);
        buffer.validate_position(&dest);

        let keep_anchor = matches!(editor_mode, EditorMode::Visual);
        self.move_to(
            dest,
            MoveOpts {
                anchor: if keep_anchor {
                    Some(self.anchor)
                } else {
                    Some(start)
                },
                update_preferred_col: true,
            },
            buffer,
        )
    }

    pub fn move_word_end(
        &mut self,
        buffer: &Buffer,
        big_word: bool,
        editor_mode: &EditorMode,
    ) -> Option<TextPosition> {
        let total_chars = buffer.content.len_chars();
        let initial_pos = self.position();

        buffer.validate_position(&initial_pos);

        let line_start = buffer.content.line_to_char(initial_pos.line);
        let mut char_idx = line_start + initial_pos.col;

        if char_idx >= total_chars {
            return None;
        }

        // Move forward one character if possible.
        if char_idx + 1 < total_chars {
            char_idx += 1;
        } else {
            // We're at the end of the buffer.
            return None;
        }

        // Skip over whitespace.
        while char_idx < total_chars {
            let c = buffer.content.char(char_idx);
            if get_char_class(c, big_word) == CharClass::Whitespace {
                char_idx += 1;
            } else {
                break;
            }
        }

        if char_idx >= total_chars {
            return None;
        }

        let current_class = get_char_class(buffer.content.char(char_idx), big_word);
        let mut last_char_index = char_idx;

        // Move to the end of the current class sequence.
        while char_idx < total_chars {
            let c = buffer.content.char(char_idx);
            if get_char_class(c, big_word) == current_class {
                last_char_index = char_idx;
                char_idx += 1;
            } else {
                break;
            }
        }

        // Convert char index back to TextPosition.
        let new_line = buffer.content.char_to_line(last_char_index);
        let new_line_start = buffer.content.line_to_char(new_line);
        let new_col = last_char_index - new_line_start;
        let new_pos = TextPosition::new(new_line, new_col, last_char_index);

        buffer.validate_position(&new_pos);
        let keep_anchor = matches!(editor_mode, EditorMode::Visual);
        self.move_to(
            new_pos,
            MoveOpts {
                anchor: if keep_anchor {
                    Some(self.anchor)
                } else {
                    Some(initial_pos)
                },
                update_preferred_col: true,
            },
            buffer,
        );

        Some(new_pos)
    }

    /// Move the cursor to `dest`, optionally extend / collapse selection and update `preferred_col`.
    ///
    /// Returns the clamped position that was finally reached (or `None` if the move is impossible - e.g.
    /// off the left edge of the buffer).
    pub fn move_to(
        &mut self,
        dest: TextPosition,
        opts: MoveOpts,
        buffer: &Buffer,
    ) -> Option<TextPosition> {
        buffer.validate_position(&dest);

        let line = dest.line.min(buffer.content.len_lines().saturating_sub(1));
        let col = dest.col.min(buffer.grapheme_len(line));
        let off = buffer.grapheme_col_to_offset(line, col);
        let dest = TextPosition::new(line, col, off);
        buffer.validate_position(&dest);

        self.active = dest;
        self.anchor = opts.anchor.unwrap_or(dest);
        if opts.update_preferred_col {
            self.preferred_column = Some(dest.col);
        }

        Some(dest)
    }

    //
    // Helpers
    //

    pub fn adjust_for_mode(&mut self, buffer: &Buffer, editor_mode: &EditorMode) {
        match editor_mode {
            EditorMode::Normal => {
                let cur = self.position();
                let line_len = buffer.grapheme_len(cur.line);
                if line_len > 0 && cur.col >= line_len {
                    // Move cursor back to the last character.
                    let new_col = line_len - 1;
                    let new_off = buffer.grapheme_col_to_offset(cur.line, new_col);
                    let new_pos = TextPosition::new(cur.line, new_col, new_off);
                    self.move_to(
                        new_pos,
                        MoveOpts {
                            anchor: None,
                            update_preferred_col: true,
                        },
                        buffer,
                    );
                }
            }
            // We only care for Normal mode here.
            _ => {}
        }
    }

    fn get_max_col(&self, editor_mode: &EditorMode, buffer: &Buffer, target: usize) -> usize {
        match editor_mode {
            EditorMode::Normal | EditorMode::Visual => {
                let line_len = buffer.grapheme_len(target);
                if line_len == 0 {
                    0
                } else {
                    line_len - 1
                }
            }
            EditorMode::Insert => buffer.grapheme_len(target),
        }
    }
}

impl TextPosition {
    /// Create a new position from line and column.
    pub fn new(line: usize, col: usize, offset: usize) -> Self {
        Self { line, col, offset }
    }
}


//
// NOTE: These tests are here to basically test how our movement logic is working compared to helix's.
// The goal is to "feel" comparable, but does not need to be exactly the same thing.
// 

#[cfg(test)]
mod helix_parity {
    use super::*;
    use crate::buffer::Buffer;
    use crate::EditorMode;

    /// Debug test to understand word movement.
    #[test]
    fn debug_word_movement() {
        let text = "int main() {\n    printf(\"hello world\");\n}\n";
        let buffer = Buffer::new(text, "debug test");
        
        let mut cursor = Cursor::new();
        cursor.move_to(
            TextPosition::new(0, 7, 7), // Position at 'n' in "main".
            MoveOpts {
                anchor: None,
                update_preferred_col: true,
            },
            &buffer,
        );
        
        // Try the word movement.
        let result = cursor.move_word_forward(&buffer, false, &EditorMode::Normal);
        
        // Verify we moved from 'n' to the space after ')'.
        assert!(result.is_some());
        let pos = cursor.position();
        assert_eq!(pos.line, 0);
        assert_eq!(pos.col, 10); // Space after ')'.
    }

    #[test]
    fn visual_w_skips_leading_punct() {
        let buffer  = Buffer::new("#include <stdio.h>", "t");
        let mut cur = Cursor::new();

        // put cursor on ‘#’, enter visual mode (anchor == active here)
        cur.move_to(TextPosition::new(0, 0, 0),
                    MoveOpts { anchor: None, update_preferred_col: true },
                    &buffer);

        // pretend we are in Visual mode and press ‘w’
        cur.move_word_forward(&buffer, false, &EditorMode::Visual);

        let (sel_start, sel_end) = cur.get_selection_range();
        assert_eq!(buffer.content.slice(sel_start.offset .. sel_end.offset).to_string(),
                   "include");
    }

    /// Helix-compatible forward-word motion (`w`) over
    /// 
    /// ```c
    ///      #include <stdio.h>
    /// 
    ///      int main() {
    ///          printf("hello world");
    ///      }
    /// ````
    ///
    /// Starting from the # character all the way to the end and marking each point,
    /// checking against helix.
    #[test]
    fn w_motion_matches_helix() {
        let text = concat!(
            "#include <stdio.h>\n",
            "\n",
            "int main() {\n",
            "\tprintf(\"hello world\");\n", // Real tab before printf.
            "}\n",
        );
        let buffer = Buffer::new(text, "helix-w-include test");

        let mut cursor = Cursor::new();
        cursor.move_to(
            TextPosition::new(0, 0, 0),
            MoveOpts { anchor: None, update_preferred_col: true },
            &buffer,
        );

        // Expected (line, col) after each press of 'w'.
        let expected: &[(usize, usize)] = &[
            (0,  8),  // space between 'e' and '<'.
            (0,  9),  // '<'.
            (0, 14),  // 'o'  in "stdio".
            (0, 15),  // '.'.
            (0, 16),  // 'h'.
            (0, 17),  // '>'.
            (2,  3),  // space between 't' and 'm'.
            (2,  7),  // 'n' (last char of "main").
            (2, 10),  // space between ')' and '{'.
            (2, 11),  // '{'.
            (3,  0),  // indent (tab).
            (3,  6),  // 'f' (last char of "printf").
            (3,  8),  // '"'  after '('.
            (3, 14),  // space between 'o' and 'w'.
            (3, 19),  // 'd'.
            (3, 22),  // ';'.
            (4,  0),  // '}'.
        ];

        for (step, &(line, col)) in expected.iter().enumerate() {
            cursor
                .move_word_forward(&buffer, /*big_word=*/ false, &EditorMode::Normal)
                .expect("`w` motion failed");

            let pos = cursor.position();
            assert_eq!(
                (pos.line, pos.col),
                (line, col),
                "step {}: expected cursor at ({}, {}), got ({}, {})",
                step + 1,
                line, col,
                pos.line, pos.col
            );
        }
    }

    /// Helix-compatible backward-word motion (`b`) over the text
    /// 
    /// ```c
    ///     #include <stdio.h>
    ///
    ///     int main() {
    ///         printf("hello world");
    ///     }
    /// ```
    /// Same thing as the above test, but starts from } and iterates by pressing "b".
    #[test]
    fn b_motion_matches_helix() {
        let text = concat!(
            "#include <stdio.h>\n",
            "\n",
            "int main() {\n",
            "    printf(\"hello world\");\n",
            "}\n",
        );
        let buffer = Buffer::new(text, "helix-b test");

        // Place the cursor on the closing ‘}’.
        let mut cursor = Cursor::new();
        let start_off = buffer.grapheme_col_to_offset(4, 0);
        cursor.move_to(
            TextPosition::new(4, 0, start_off),
            MoveOpts { anchor: None, update_preferred_col: true },
            &buffer,
        );

        let expected: &[(usize, usize)] = &[
            (3, 23), // '"'    after world.
            (3, 18), // 'w'.
            (3, 12), // 'h'.
            (3, 10), // '('.
            (3,  4), // 'p'.
            (3,  0), // indent (first space).
            (2, 11), // '{'.
            (2,  8), // '('.
            (2,  4), // 'm'.
            (2,  0), // 'i'.
            (0, 17), // '>'.
            (0, 16), // 'h'.
            (0, 15), // '.'.
            (0, 10), // 's'.
            (0,  9), // '<'.
            (0,  1), // 'i'.
            (0,  0), // '#'.
        ];

        for (step, &(line, col)) in expected.iter().enumerate() {
            let start_pos = cursor.position();
            let start_char = if start_pos.offset < buffer.content.len_chars() {
                buffer.content.char(start_pos.offset)
            } else {
                '\0'
            };
            println!("Step {}: Starting from ({},{}) char='{}' offset={}", 
                     step + 1, start_pos.line, start_pos.col, 
                     if start_char == '\n' { '\\' } else { start_char }, 
                     start_pos.offset);
            
            cursor
                .move_word_backward(&buffer, /*big_word=*/ false, &EditorMode::Normal)
                .expect("`b` motion failed");

            let pos = cursor.position();
            let end_char = if pos.offset < buffer.content.len_chars() {
                buffer.content.char(pos.offset)
            } else {
                '\0'
            };
            println!("Step {}: Ended at ({},{}) char='{}' offset={}", 
                     step + 1, pos.line, pos.col, 
                     if end_char == '\n' { '\\' } else { end_char }, 
                     pos.offset);
            
            assert_eq!(
                (pos.line, pos.col),
                (line, col),
                "step {}: expected cursor at ({}, {}), got ({}, {})",
                step + 1,
                line, col,
                pos.line, pos.col
            );
        }
    }
}
