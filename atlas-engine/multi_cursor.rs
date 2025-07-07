use crate::{cursor::MoveOpts, Buffer, Cursor, EditorMode, TextPosition};

/// A collection of `Cursor` objects that are moved/edited together.
///
/// The first cursor in `cursors` is considered the *primary* cursor.
/// All UI-related state (for example where a new cursor is added) is
/// derived from the primary cursor.
#[derive(Debug, Clone)]
pub struct MultiCursor {
    pub cursors: Vec<Cursor>,
    pub primary_index: usize,
}

impl Default for MultiCursor {
    fn default() -> Self {
        Self::new()
    }
}

macro_rules! generate_cursor_methods {
    // With buffer params.
    ($method_name:ident($buffer_param:ident: &Buffer)) => {
        pub fn $method_name(&mut self, $buffer_param: &Buffer) {
            for cursor in &mut self.cursors {
                cursor.$method_name($buffer_param);
            }
            self.merge_overlapping();
        }
    };

    // Buffer + additional params.
    ($method_name:ident($buffer_param:ident: &Buffer, $($param:ident: $param_type:ty),+)) => {
        pub fn $method_name(&mut self, $buffer_param: &Buffer, $($param: $param_type),+) {
            for cursor in &mut self.cursors {
                cursor.$method_name($buffer_param, $($param),+);
            }
            self.merge_overlapping();
        }
    };

    // Method that doesn't need merge_overlapping (like adjust_for_mode).
    (no_merge $method_name:ident($buffer_param:ident: &Buffer, $($param:ident: $param_type:ty),+)) => {
        pub fn $method_name(&mut self, $buffer_param: &Buffer, $($param: $param_type),+) {
            for cursor in &mut self.cursors {
                cursor.$method_name($buffer_param, $($param),+);
            }
        }
    };
}

impl MultiCursor {
    /// Create a new `MultiCursor` with a single cursor at position (0,0).
    pub fn new() -> Self {
        Self {
            cursors: vec![Cursor::new()],
            primary_index: 0,
        }
    }

    //
    // Simple accessors
    //

    /// Immutable access to all cursors.
    pub fn all_cursors(&self) -> &[Cursor] {
        &self.cursors
    }

    /// Mutable access to all cursors.
    pub fn all_cursors_mut(&mut self) -> &mut [Cursor] {
        &mut self.cursors
    }

    /// Reference to the *primary* cursor.
    pub fn primary(&self) -> &Cursor {
        &self.cursors[self.primary_index]
    }

    /// Mutable reference to the *primary* cursor.
    pub fn primary_mut(&mut self) -> &mut Cursor {
        &mut self.cursors[self.primary_index]
    }

    /// Convenience helper – current position of the *primary* cursor.
    pub fn position(&self) -> TextPosition {
        self.primary().position()
    }

    //
    // Cursor manipulation helpers
    //

    /// Apply a closure to **all** cursors. Any modification is followed by a
    /// call to `merge_overlapping` to guarantee the invariant that no two
    /// cursors occupy the same text position.
    pub fn apply_to_all<F>(&mut self, mut f: F)
    where
        F: FnMut(&mut Cursor),
    {
        // NOTE: Process cursors from *right to left* (higher offsets first). This way any change to the
        // buffer (insert/delete) will not invalidate the still-unprocessed cursors because their
        // offsets are <= the current mutation point.
        let mut indices: Vec<usize> = (0..self.cursors.len()).collect();
        indices.sort_by_key(|&i| self.cursors[i].position().offset);

        for i in indices.into_iter().rev() {
            f(&mut self.cursors[i]);
        }
        
        // Finally, deduplicate/merge and keep invariants.
        self.merge_overlapping();

        // After edits the offsets of other cursors may have become stale. Re-calculate them.
        // SAFETY: we need the buffer reference, so `apply_to_all` must be called from contexts
        // where the buffer is available; therefore we will expose a different helper that is
        // called by the public API instead. For now, we omit automatic refresh here.
    }

    /// Add a new cursor at the provided text position.
    pub fn add_cursor(&mut self, pos: TextPosition, buffer: &Buffer) {
        let mut cursor = Cursor::new();
        cursor.move_to(pos, MoveOpts { anchor: None, update_preferred_col: false}, buffer);
        self.cursors.push(cursor);
        self.merge_overlapping();
    }

    /// Remove every cursor except for the primary cursor.
    pub fn clear_secondary_cursors(&mut self) {
        if self.cursors.len() > 1 {
            let primary_cursor = self.primary().clone();
            self.cursors.clear();
            self.cursors.push(primary_cursor);
            self.primary_index = 0;
        }
    }

    //
    // Movement helpers.
    // Broadcast to all cursors.
    //

    generate_cursor_methods!(move_left(buffer: &Buffer, mode: &EditorMode));
    generate_cursor_methods!(move_right(buffer: &Buffer, mode: &EditorMode));
    generate_cursor_methods!(move_up(buffer: &Buffer, mode: &EditorMode));
    generate_cursor_methods!(move_down(buffer: &Buffer, mode: &EditorMode));
    generate_cursor_methods!(move_word_forward(buffer: &Buffer, big_word: bool, mode: &EditorMode));
    generate_cursor_methods!(move_word_backward(buffer: &Buffer, big_word: bool, mode: &EditorMode));
    generate_cursor_methods!(move_word_end(buffer: &Buffer, big_word: bool, mode: &EditorMode));

    generate_cursor_methods!(no_merge adjust_for_mode(buffer: &Buffer, mode: &EditorMode));

    /// After any mutation we call this function to ensure we do not have two
    /// cursors in exactly the same location. If that happens we keep the
    /// first one and delete the others. The *primary* cursor is preserved
    /// regardless – we only update its index to the new position after the
    /// vector is compacted.
    fn merge_overlapping(&mut self) {
        if self.cursors.len() <= 1 {
            return;
        }

        let primary_offset = self.primary().position().offset;

        self.cursors.sort_by_key(|c| c.position().offset);

        self.cursors.dedup_by_key(|c| c.position().offset);

        // Restore `primary_index`.
        if let Some(idx) = self
            .cursors
            .iter()
            .position(|c| c.position().offset == primary_offset)
        {
            self.primary_index = idx;
        } else {
            self.primary_index = 0;
        }
    }

    /// Ensure every cursor's `offset` matches its `(line,col)` after arbitrary buffer edits.
    /// PERFORMANCE: Expensive (O(n)) but cheap enough given few cursors, which should be the majority of use cases.
    pub fn refresh_positions(&mut self, buffer: &Buffer) {
        for cursor in &mut self.cursors {
            let pos = cursor.position();
            let correct_offset = buffer.grapheme_col_to_offset(pos.line, pos.col);
            if pos.offset != correct_offset {
                let new_pos = TextPosition::new(pos.line, pos.col, correct_offset);
                cursor.move_to(new_pos, MoveOpts { anchor: None, update_preferred_col: false }, buffer);
            }
        }
    }
}
