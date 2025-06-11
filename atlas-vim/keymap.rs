use std::collections::HashMap;

use atlas_engine::VimMode;

use crate::{Motion, Operator, VimAction};

#[derive(Clone, Eq, Hash, PartialEq)]
pub enum KeyAction {
    KeyMotion(Motion),
    KeyOperator(Operator),
    Command(VimAction),
    Custom(fn() -> VimAction),
}

#[derive(Clone)]
pub struct Keymap {
    bindings: HashMap<(VimMode, String), KeyAction>,
    multi_key_buffer: String,
}

impl Keymap {
    pub fn new() -> Self {
        let mut keymap = Self {
            bindings: HashMap::new(),
            multi_key_buffer: String::new(),
        };

        // Set up default vim bindings.
        keymap.setup_defaults();
        keymap
    }

    pub fn set(&mut self, mode: VimMode, keys: &str, action: KeyAction) {
        self.bindings.insert((mode, keys.to_string()), action);
    }

    pub fn handle_key(
        &mut self,
        mode: &VimMode,
        key: char,
        count: Option<usize>,
    ) -> Option<VimAction> {
        self.multi_key_buffer.push(key);

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

    fn create_action(&self, action: &KeyAction, count: usize) -> VimAction {
        match action {
            KeyAction::KeyMotion(motion) => VimAction::Move {
                motion: motion.clone(),
                count,
            },
            KeyAction::KeyOperator(_) => {
                // NOTE: This would be handled differently - operators need motions.
                todo!("Handle operators with keymap")
            }
            KeyAction::Command(cmd) => cmd.clone(),
            KeyAction::Custom(func) => func(),
        }
    }

    fn setup_defaults(&mut self) {
        use KeyAction::*;
        use VimMode::*;

        // Basic movements.
        self.set(Normal, "h", KeyMotion(Motion::CharLeft));
        self.set(Normal, "j", KeyMotion(Motion::CharDown));
        self.set(Normal, "k", KeyMotion(Motion::CharUp));
        self.set(Normal, "l", KeyMotion(Motion::CharRight));

        // Word movements.
        self.set(Normal, "w", KeyMotion(Motion::NextWordStart(false)));
        self.set(Normal, "W", KeyMotion(Motion::NextWordStart(true)));
        self.set(Normal, "b", KeyMotion(Motion::PrevWord(false)));
        self.set(Normal, "B", KeyMotion(Motion::PrevWord(true)));
        self.set(Normal, "e", KeyMotion(Motion::NextWordEnd(false)));
        self.set(Normal, "E", KeyMotion(Motion::NextWordEnd(true)));

        // Mode changes.
        self.set(Normal, "i", Command(VimAction::ChangeMode(Insert)));
        self.set(Normal, "v", Command(VimAction::ChangeMode(Visual)));

        // Other commands.
        self.set(Normal, "x", Command(VimAction::Delete));
        self.set(Normal, ".", Command(VimAction::RepeatLast));

        // Operators.
        self.set(Normal, "d", KeyOperator(Operator::Delete));
        self.set(Normal, "y", KeyOperator(Operator::Yank));
        self.set(Normal, "c", KeyOperator(Operator::Change));

        // Multi-key bindings.
        self.set(
            Normal,
            "gg",
            Custom(|| VimAction::Move {
                motion: Motion::ToLineStart,
                count: 1,
            }),
        );

        // A taste of the future.
        // self.set(Normal, "gd", Custom(go_to_definition));
        // self.set(Normal, "gr", Custom(replace_under_cursor));
    }
}
