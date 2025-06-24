use atlas_engine::{Buffer, VimMode, MultiCursor, Message};
use iced::keyboard::{self, Key, Modifiers};

use crate::keymap::Keymap;

#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub enum Motion {
    CharLeft,
    CharRight,
    CharUp,
    CharDown,
    ToLineStart,
    _ToLineEnd,
    NextWordStart(bool), // NOTE: Boolean value to represent if it's a big word or not.
    NextWordEnd(bool),
    PrevWord(bool),
}

impl Motion {
    pub fn from_hjkl(c: char) -> Option<Self> {
        Some(match c {
            'h' => Motion::CharLeft,
            'j' => Motion::CharDown,
            'k' => Motion::CharUp,
            'l' => Motion::CharRight,
            'w' => Motion::NextWordStart(false),
            'W' => Motion::NextWordStart(true),
            'b' => Motion::PrevWord(false),
            'B' => Motion::PrevWord(true),
            'e' => Motion::NextWordEnd(false),
            'E' => Motion::NextWordEnd(true),
            _ => return None,
        })
    }
}

#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub enum Operator {
    Delete,
    Yank,
    Change,
    // TODO
}

impl Operator {
    pub fn from_char(c: char) -> Option<Self> {
        Some(match c {
            'd' => Operator::Delete,
            'y' => Operator::Yank,
            'c' => Operator::Change,
            _ => return None,
        })
    }
}

#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub enum VimAction {
    InsertChar(char),
    InsertText(String),
    InsertNewline,
    Move {
        motion: Motion,
        count: usize,
    },
    Operate {
        _op: Operator, // dw, 3yy, etc.
        _motion: Motion,
        _count: usize,
    },
    ChangeMode(VimMode),
    RepeatLast,
    Backspace,
    Delete,
    AddCursor, // NOTE: This is likely just mocked.
    RemoveSecondaryCursors,
}

#[derive(Debug, Clone, PartialEq)]
pub enum EngineAction {
    Vim(VimAction),
    App(Message),
}

#[derive(Clone)]
pub struct VimEngine {
    pub mode: VimMode,
    keymap: Keymap,
    last_edit: Option<VimAction>, // For ".".
}

impl Default for VimEngine {
    fn default() -> Self {
        Self {
            mode: VimMode::Normal,
            keymap: Keymap::new(),
            last_edit: None,
        }
    }
}

impl VimEngine {
    /// Returns at most **one** high-level action for the editor to execute.
    pub fn handle_key(&mut self, key: KeyEvent) -> Option<EngineAction> {
        use VimMode::*;
        match self.mode {
            Insert => match key {
                KeyEvent::Key { key, text, .. } => {
                    // Prioritize text if available.
                    if let Some(s) = text {
                        if !s.is_empty() {
                            for ch in s.chars() {
                                self.last_edit = Some(VimAction::InsertChar(ch));
                            }
                            return Some(EngineAction::Vim(VimAction::InsertText(s)));
                        }
                    }

                    // Fallback to raw key if no text.
                    if let Key::Character(s) = key {
                        if s.len() == 1 {
                            let c = s.chars().next().unwrap();
                            if !c.is_control() {
                                self.last_edit = Some(VimAction::InsertChar(c));
                                return Some(EngineAction::Vim(VimAction::InsertChar(c)));
                            }
                        }
                    }
                    None
                }

                KeyEvent::Esc => {
                    self.mode = Normal;
                    Some(EngineAction::Vim(VimAction::ChangeMode(Normal)))
                }
                KeyEvent::Backspace => Some(EngineAction::Vim(VimAction::Backspace)),
                KeyEvent::Enter => Some(EngineAction::Vim(VimAction::InsertNewline)), // NOTE: Enter should likely be an action
            },

            Normal => {
                if let Some(action) = self.keymap.handle_key(&self.mode, &key, None) {
                    if let EngineAction::Vim(v_action) = &action {
                        if matches!(
                            v_action,
                            VimAction::InsertChar(_) | VimAction::Operate { .. }
                        ) {
                            self.last_edit = Some(v_action.clone());
                        }
                        if let VimAction::ChangeMode(m) = &v_action {
                            self.mode = m.clone();
                        }
                    }
                    return Some(action);
                }

                None
            }

            Visual => {
                if let KeyEvent::Key {
                    key: Key::Character(ref s),
                    ..
                } = key
                {
                    if s.len() == 1 {
                        let c = s.chars().next().unwrap();

                        // Handle movement in visual mode just like we do in normal mode.
                        if let Some(motion) = Motion::from_hjkl(c) {
                            return Some(EngineAction::Vim(VimAction::Move { motion, count: 1 }));
                        }

                        // Handle operators in visual mode, also just like we do it in normal mode.
                        if let Some(op) = Operator::from_char(c) {
                            // In visual mode, operators work on the selection.
                            self.mode = Normal;
                            return Some(EngineAction::Vim(VimAction::Operate {
                                _op: op,
                                _motion: Motion::CharRight, // Placeholder - will use selection.
                                _count: 1,
                            }));
                        }
                    }
                }

                if let KeyEvent::Esc = key {
                    self.mode = Normal;
                    return Some(EngineAction::Vim(VimAction::ChangeMode(Normal)));
                }

                None
            }
        }
    }

    pub fn _repeat_last(&self) -> Option<VimAction> {
        self.last_edit.clone()
    }

    /// Count handling
    pub fn has_pending_count(&self) -> bool {
        // TODO: Implement count handling in keymap.
        false
    }
}

/// A minimal key event used inside the engine.
/// The `on_event` widget methods translate iced events into this.
#[derive(Clone)]
pub enum KeyEvent {
    Key {
        key: keyboard::Key,
        text: Option<String>,
        modifiers: Modifiers,
    },
    Esc,
    Backspace,
    Enter,
}

pub fn execute(action: VimAction, buffer: &mut Buffer, multi_cursor: &mut MultiCursor, vim_mode: &VimMode) {
    match action {
        VimAction::InsertChar(c) => buffer.insert_char(multi_cursor, c),
        VimAction::InsertText(s) => buffer.insert_text(multi_cursor, s.as_str()),
        VimAction::Move { motion, .. } => apply_motion(motion, buffer, multi_cursor, vim_mode),
        VimAction::Operate { .. } => println!("Todo!"),
        VimAction::ChangeMode(new_mode) => multi_cursor.adjust_for_mode(buffer, &new_mode),
        VimAction::RepeatLast => println!("Handled by engine"),
        VimAction::Backspace => buffer.backspace(multi_cursor),
        VimAction::InsertNewline => buffer.insert_newline(multi_cursor),
        VimAction::Delete => buffer.delete(multi_cursor),
        
        // MOCKED
        VimAction::AddCursor => {
            // Add a cursor one line below the primary cursor, or to the right if at last line.
            let current_pos = multi_cursor.position();
            let total_lines = buffer.content.len_lines();

            let new_pos = if current_pos.line + 1 < total_lines {
                // Move to next line, same column (or end of line if shorter).
                let next_line = current_pos.line + 1;
                let line_len = buffer.grapheme_len(next_line);
                let new_col = current_pos.col.min(line_len);
                let new_offset = buffer.grapheme_col_to_offset(next_line, new_col);

                atlas_engine::TextPosition::new(next_line, new_col, new_offset)
            } else {
                // At last line, try to move right instead.
                let line_len = buffer.grapheme_len(current_pos.line);
                if current_pos.col < line_len {
                    let new_col = current_pos.col + 1;
                    let new_offset = buffer.grapheme_col_to_offset(current_pos.line, new_col);
                    atlas_engine::TextPosition::new(current_pos.line, new_col, new_offset)
                } else {
                    // Can't add cursor anywhere, just return without adding.
                    return;
                }
            };

            buffer.validate_position(&new_pos);
            multi_cursor.add_cursor(new_pos, buffer);
        },
        
        VimAction::RemoveSecondaryCursors => multi_cursor.clear_secondary_cursors(),
    }
}

fn apply_motion(motion: Motion, buffer: &Buffer, multi_cursor: &mut MultiCursor, vim_mode: &VimMode) {
    match motion {
        Motion::CharLeft => multi_cursor.move_left(buffer),
        Motion::CharRight => multi_cursor.move_right(buffer, vim_mode),
        Motion::CharUp => multi_cursor.move_up(buffer, vim_mode),
        Motion::CharDown => multi_cursor.move_down(buffer, vim_mode),
        Motion::NextWordStart(big_word) => multi_cursor.move_word_forward(buffer, big_word),
        Motion::PrevWord(big_word) => multi_cursor.move_word_backward(buffer, big_word),
        Motion::NextWordEnd(big_word) => multi_cursor.move_word_end(buffer, big_word),
        Motion::ToLineStart => println!("Line start"),
        Motion::_ToLineEnd => todo!(),
    }
}
