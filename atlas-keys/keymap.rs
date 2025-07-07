use std::collections::HashMap;

use atlas_engine::{Message, EditorMode};
use iced::keyboard::Key;

use crate::{EngineAction, KeyEvent, Motion, Operator, Action};

#[derive(Clone)]
pub enum KeyAction {
    KeyMotion(Motion),
    KeyOperator(Operator),
    Command(Action),
    Custom(fn() -> Action),
    AppCommand(Message),
}

#[derive(Default, Clone)]
pub struct Keymap {
    bindings: HashMap<(EditorMode, String), KeyAction>,
    multi_key_buffer: String,
}

impl Keymap {
    pub fn new() -> Self {
        let mut keymap = Self {
            bindings: HashMap::new(),
            multi_key_buffer: String::new(),
        };

        // Set up default bindings.
        keymap.setup_defaults();
        keymap
    }

    pub fn set(&mut self, mode: EditorMode, keys: &str, action: KeyAction) {
        self.bindings.insert((mode, keys.to_string()), action);
    }

    pub fn handle_key(
        &mut self,
        mode: &EditorMode,
        key: &KeyEvent,
        count: Option<usize>,
    ) -> Option<EngineAction> {
        let key_str = self.key_to_string(key);
        if key_str.is_empty() {
            return None;
        }
        self.multi_key_buffer.push_str(&key_str);

        // Check for exact match.
        if let Some(action) = self
            .bindings
            .get(&(mode.clone(), self.multi_key_buffer.clone()))
        {
            let result = self.create_action(action, count.unwrap_or(1));
            self.multi_key_buffer.clear();
            return Some(result);
        }

        let has_partial = self
            .bindings
            .keys()
            .any(|(m, keys)| m == mode && keys.starts_with(&self.multi_key_buffer));

        if !has_partial {
            self.multi_key_buffer.clear();
            None
        } else {
            None
        }
    }

    fn key_to_string(&self, key: &KeyEvent) -> String {
        if let KeyEvent::Key { key, modifiers, .. } = key {
            let mut s = String::new();

            let key_char = match key.as_ref() {
                Key::Character(c) => Some(c.to_lowercase()),
                _ => None,
            };

            if modifiers.control() {
                s.push_str("<C-");
            }
            if modifiers.alt() {
                s.push_str("<A-");
            }
            if modifiers.shift() {
                s.push_str("<S-");
            }

            if let Some(c) = key_char {
                s.push_str(&c);
            }

            if s.len() > 1 && s.starts_with('<') {
                s.push('>');
            }

            if s.len() <= 2 && s.ends_with('>') {
                return String::new();
            }

            s
        } else {
            String::new()
        }
    }

    fn create_action(&self, action: &KeyAction, count: usize) -> EngineAction {
        match action {
            KeyAction::KeyMotion(motion) => EngineAction::Action(Action::Move {
                motion: motion.clone(),
                count,
            }),
            KeyAction::KeyOperator(_) => {
                // NOTE: This would be handled differently - operators need motions.
                todo!("Handle operators with keymap")
            }
            KeyAction::Command(cmd) => EngineAction::Action(cmd.clone()),
            KeyAction::Custom(func) => EngineAction::Action(func()),
            KeyAction::AppCommand(msg) => EngineAction::App(msg.clone()),
        }
    }

    fn setup_defaults(&mut self) {
        use KeyAction::*;
        use EditorMode::*;

        // Basic movements.
        self.set(Normal, "h", KeyMotion(Motion::CharLeft));
        self.set(Normal, "j", KeyMotion(Motion::CharDown));
        self.set(Normal, "k", KeyMotion(Motion::CharUp));
        self.set(Normal, "l", KeyMotion(Motion::CharRight));

        // Word movements.
        self.set(Normal, "w", KeyMotion(Motion::NextWordStart(false)));
        self.set(Normal, "<S-w>", KeyMotion(Motion::NextWordStart(true)));
        self.set(Normal, "b", KeyMotion(Motion::PrevWord(false)));
        self.set(Normal, "<S-b>", KeyMotion(Motion::PrevWord(true)));
        self.set(Normal, "e", KeyMotion(Motion::NextWordEnd(false)));
        self.set(Normal, "<S-e>", KeyMotion(Motion::NextWordEnd(true)));

        // Mode changes.
        self.set(Normal, "i", Command(Action::ChangeMode(Insert)));
        self.set(Normal, "v", Command(Action::ChangeMode(Visual)));

        // Other commands.
        self.set(Normal, "x", Command(Action::Delete));
        self.set(Normal, ".", Command(Action::RepeatLast));

        // Operators.
        self.set(Normal, "d", KeyOperator(Operator::Delete));
        self.set(Normal, "y", KeyOperator(Operator::Yank));
        self.set(Normal, "c", KeyOperator(Operator::Change));

        // Testing multiple cursors.
        self.set(Normal, "<S-c>", Command(Action::AddCursor));
        self.set(Normal, "<S-r>", Command(Action::RemoveSecondaryCursors));

        // Window splitting.
        self.set(Normal, "<C-v>", AppCommand(Message::SplitVertical));
        self.set(Normal, "<C-h>", AppCommand(Message::SplitHorizontal));
        self.set(Normal, "<C-w>", AppCommand(Message::CloseSplit));

        // Quit atlas.
        self.set(Normal, "<C-q>", AppCommand(Message::Quit));

        // Multi-key bindings.
        self.set(
            Normal,
            "gg",
            Custom(|| Action::Move {
                motion: Motion::ToLineStart,
                count: 1,
            }),
        );

        // A taste of the future.
        // self.set(Normal, "gd", Custom(go_to_definition));
        // self.set(Normal, "gr", Custom(replace_under_cursor));
    }
}
