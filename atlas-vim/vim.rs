use atlas_engine::{Buffer, Cursor, VimMode};
use iced::keyboard::{self, Key};

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
    pub fn handle_key(&mut self, key: KeyEvent) -> Option<VimAction> {
        use VimMode::*;
        match self.mode {
            Insert => match key {
                KeyEvent::Key { key, text } => {
                    // Prioritize text if available.
                    if let Some(s) = text {
                        if !s.is_empty() {
                            for ch in s.chars() {
                                self.last_edit = Some(VimAction::InsertChar(ch));
                            }
                            return Some(VimAction::InsertText(s));
                        }
                    }

                    // Fallback to raw key if no text.
                    if let Key::Character(s) = key {
                        if s.len() == 1 {
                            let c = s.chars().next().unwrap();
                            if !c.is_control() {
                                self.last_edit = Some(VimAction::InsertChar(c));
                                return Some(VimAction::InsertChar(c));
                            }
                        }
                    }
                    None
                }

                KeyEvent::Esc => {
                    self.mode = Normal;
                    Some(VimAction::ChangeMode(Normal))
                }
                KeyEvent::Backspace => Some(VimAction::Backspace),
                KeyEvent::Enter => Some(VimAction::InsertNewline), // NOTE: Enter should likely be an action
            },

            Normal => {
                if let KeyEvent::Key { key, .. } = key {
                    if let Key::Character(s) = key {
                        if s.len() == 1 {
                            let c = s.chars().next().unwrap();
                            if let Some(action) = self.keymap.handle_key(&self.mode, c, None) {
                                if matches!(
                                    action,
                                    VimAction::InsertChar(_) | VimAction::Operate { .. }
                                ) {
                                    self.last_edit = Some(action.clone());
                                }
                                if let VimAction::ChangeMode(m) = &action {
                                    self.mode = m.clone();
                                }
                                return Some(action);
                            }
                        }
                    }
                }

                None
            }

            Visual => {
                if let KeyEvent::Key { ref key, .. } = key {
                    if let Key::Character(s) = key {
                        if s.len() == 1 {
                            let c = s.chars().next().unwrap();

                            // Handle movement in visual mode just like we do in normal mode.
                            if let Some(motion) = Motion::from_hjkl(c) {
                                return Some(VimAction::Move { motion, count: 1 });
                            }

                            // Handle operators in visual mode, also just like we do it in normal mode.
                            if let Some(op) = Operator::from_char(c) {
                                // In visual mode, operators work on the selection.
                                self.mode = Normal;
                                return Some(VimAction::Operate {
                                    _op: op,
                                    _motion: Motion::CharRight, // Placeholder - will use selection.
                                    _count: 1,
                                });
                            }
                        }
                    }
                }

                if let KeyEvent::Esc = key {
                    self.mode = Normal;
                    return Some(VimAction::ChangeMode(Normal));
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
    },
    Esc,
    Backspace,
    Enter,
}

pub fn execute(action: VimAction, buffer: &mut Buffer, cursor: &mut Cursor, vim_mode: &VimMode) {
    match action {
        VimAction::InsertChar(c) => buffer.insert_char(cursor, c),
        VimAction::InsertText(s) => buffer.insert_text(cursor, s.as_str()),
        VimAction::Move { motion, .. } => apply_motion(motion, buffer, cursor, vim_mode),
        VimAction::Operate { .. } => println!("Todo!"),
        VimAction::ChangeMode(new_mode) => {
            // Adjust cursor position when switching modes
            cursor.adjust_for_mode(buffer, &new_mode);
        }
        VimAction::RepeatLast => println!("Handled by engine"),
        VimAction::Backspace => buffer.backspace(cursor),
        VimAction::InsertNewline => buffer.insert_newline(cursor),
        VimAction::Delete => buffer.delete(cursor),
    }
}

fn apply_motion(motion: Motion, buffer: &Buffer, cursor: &mut Cursor, vim_mode: &VimMode) {
    match motion {
        Motion::CharLeft => {
            cursor.move_left(buffer);
        }
        Motion::CharRight => {
            cursor.move_right(buffer, vim_mode);
        }
        Motion::CharUp => {
            cursor.move_up(buffer, vim_mode);
        }
        Motion::CharDown => {
            cursor.move_down(buffer, vim_mode);
        }
        Motion::NextWordStart(big_word) => {
            cursor.move_word_forward(buffer, big_word);
        }
        Motion::PrevWord(big_word) => {
            cursor.move_word_backward(buffer, big_word);
        }
        Motion::NextWordEnd(big_word) => {
            cursor.move_word_end(buffer, big_word);
        }
        Motion::ToLineStart => {
            println!("Line start");
        }
        Motion::_ToLineEnd => {
            println!("Line end");
        }
    }
}
