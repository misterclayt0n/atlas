use iced::keyboard::{self, Key};

use crate::buffer::Buffer;
use crate::cursor::Cursor;

#[derive(Clone)]
pub enum Motion {
    CharLeft,
    CharRight,
    CharUp,
    CharDown,
    ToLineStart,
    _ToLineEnd,
    NextWordStart(bool), // NOTE: Boolean value to represent if it's a big word or not.
}

impl Motion {
    pub fn from_hjkl(c: char) -> Option<Self> {
        Some(match c {
            'h' => Motion::CharLeft,
            'j' => Motion::CharDown,
            'k' => Motion::CharUp,
            'l' => Motion::CharRight,
            'w' => Motion::NextWordStart(false),
            // FIX: 'W' ain't working.
            'W' => Motion::NextWordStart(true),
            _ => return None,
        })
    }
}

#[derive(Debug, Clone)]
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

#[derive(Clone)]
pub enum VimAction {
    InsertChar(char),
    InsertText(String),
    InsertNewline,
    Move {
        motion: Motion,
        _count: usize,
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

#[derive(Debug, Clone, PartialEq)]
pub enum VimMode {
    Normal,
    Insert,
    // Visual,
}

#[derive(Default, Clone)]
pub struct Parser {
    // Pending pieces for a Normal mode command.
    count: Option<usize>,
    pending_op: Option<Operator>,
}

impl Parser {
    /// Feed a single key press and maybe get in return some vim action.
    pub fn feed_key(&mut self, c: char) -> Option<VimAction> {
        // Handle counts.
        if c.is_ascii_digit() && !(c == '0' && self.count.is_none()) {
            let d = c.to_digit(10).unwrap() as usize; // UNSAFE
            self.count = Some(self.count.unwrap_or(0) * 10 + d);
            return None;
        }

        // Operator?
        if let Some(op) = Operator::from_char(c) {
            // Repeat 'dd' : second 'd' completes command using implicit line motion.
            if let Some(pending) = self.pending_op.take() {
                println!("dd motherfucker");
                let count = self.count.take().unwrap_or(1);
                return Some(VimAction::Operate {
                    _op: pending,
                    _motion: Motion::ToLineStart, // Shorthand: dd == d d
                    _count: count,
                });
            }

            self.pending_op = Some(op);
            return None;
        }

        // Motions
        if let Some(motion) = Motion::from_hjkl(c) {
            if let Some(op) = self.pending_op.take() {
                let count = self.count.take().unwrap_or(1);
                return Some(VimAction::Operate { _op: op, _motion: motion, _count: count });
            }

            let count = self.count.take().unwrap_or(1);
            return Some(VimAction::Move { motion, _count: count });
        }

        // Insert / Esc.
        match c {
            'i' => return Some(VimAction::ChangeMode(VimMode::Insert)),
            // Esc
            '\u{1b}' => return Some(VimAction::ChangeMode(VimMode::Normal)),
            '.' => return Some(VimAction::RepeatLast),
            'x' => return Some(VimAction::Delete),
            _ => {}
        }

        self.reset();
        None
    }

    pub fn reset(&mut self) {
        self.count = None;
        self.pending_op = None;
    }
}

#[derive(Clone)]
pub struct VimEngine {
    pub mode: VimMode,
    parser: Parser,
    last_edit: Option<VimAction>, // For ".".
}

impl Default for VimEngine {
    fn default() -> Self {
        Self {
            mode: VimMode::Normal,
            parser: Parser::default(),
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
                            if let Some(action) = self.parser.feed_key(c) {
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
        }
    }

    pub fn _repeat_last(&self) -> Option<VimAction> {
        self.last_edit.clone()
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

pub fn execute(action: VimAction, buffer: &mut Buffer, cursor: &mut Cursor) {
    match action {
        VimAction::InsertChar(c) => buffer.insert_char(cursor, c),
        VimAction::InsertText(s) => buffer.insert_text(cursor, s.as_str()),
        VimAction::Move { motion, .. } => apply_motion(motion, buffer, cursor),
        VimAction::Operate { .. } => println!("Todo!"),
        VimAction::ChangeMode(_) | VimAction::RepeatLast => println!("Handled by engine"),
        VimAction::Backspace => buffer.backspace(cursor),
        VimAction::InsertNewline => buffer.insert_newline(cursor),
        VimAction::Delete => buffer.delete(cursor)
    }
}

fn apply_motion(motion: Motion, buffer: &Buffer, cursor: &mut Cursor) {
    match motion {
        Motion::CharLeft => {
            cursor.move_left(buffer);
        }
        Motion::CharRight => {
            cursor.move_right(buffer);
        }
        Motion::CharUp => {
            cursor.move_up(buffer);
        }
        Motion::CharDown => {
            cursor.move_down(buffer);
        }
        Motion::NextWordStart(big_word) => {
            cursor.move_word_forward(buffer, big_word);
        }
        Motion::ToLineStart => {
            println!("Line start");
        }
        Motion::_ToLineEnd => {
            println!("Line end");
        } // …
    }
}
