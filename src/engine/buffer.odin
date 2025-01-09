// TODO: Proper error handling. For now, I'm basicaly just ignoring errors.
package engine

import rl "vendor:raylib"

Buffer :: struct {
	bytes:       [dynamic]u8,  // Dynamic array of bytes that containts text.
	line_starts: [dynamic]int, // Indexes of the beginning of each line in the array byte.
	dirty:       bool,         // If the buffer has been modified.
	cursor:      Cursor,
}

Cursor :: struct {
	pos:   int,         // Position in the array of bytes.
	sel:   int,         // Position of selection.
	line:  int,         // Current line number.
	col:   int,         // Current column number.
	style: CursorStyle, // Visual style of the cursor.
	color: rl.Color,    // Color of the cursor.
	blink: bool,        // If it blinks or not.
}

CursorStyle :: enum {
	Bar,
	Block,
	Underscore,
}

CursorMovement :: enum {
	Left,
	Right,
	Up,
	Down,
	LineStart,
	LineEnd,
	WordLeft,
	WordRight,
}

buffer_init :: proc(font: ^rl.Font, allocator := context.allocator) -> Buffer {
	return Buffer {
		bytes = make([dynamic]u8, 0, 1024, allocator),
		line_starts = make([dynamic]int, 1, 64, allocator),
		dirty = false,
		cursor = Cursor {
			pos = 0,
			sel = 0,
			line = 0,
			col = 0,
			style = .Block,
			color = rl.BLACK,
			blink = false,
		},
	}
}

// NOTE: This kind of becomes useless if we're using an arena, but it's still nice
// to have.
buffer_free :: proc(buffer: ^Buffer) {
	delete(buffer.bytes)
	delete(buffer.line_starts)
}

//
// Editing
//

// Inserts a string at the current position of the cursor.
buffer_insert_text :: proc(buffer: ^Buffer, text: string) {
	if len(text) == 0 do return
	offset := buffer.cursor.pos
	if offset < 0 || offset > len(buffer.bytes) do return
	text_bytes := transmute([]u8)text

	// Make space for new text.
	resize(&buffer.bytes, len(buffer.bytes) + len(text_bytes))

	// Move existing text to make room.
	if len(buffer.bytes) - len(text_bytes) > offset {
		copy(buffer.bytes[offset + len(text_bytes):], buffer.bytes[offset:])
	}

	// Insert new text.
	copy(buffer.bytes[offset:], text_bytes)
	buffer.cursor.pos += len(text_bytes)
	buffer.dirty = true
	buffer_update_line_starts(buffer)
}

buffer_insert_char :: proc(buffer: ^Buffer, char: rune) {
	if char < 32 || char >= 127 do return

	offset := buffer.cursor.pos
	if offset < 0 || offset > len(buffer.bytes) do return

	// Make space for new character.
	resize(&buffer.bytes, len(buffer.bytes) + 1)

	// Move existing text to make room.
	if offset < len(buffer.bytes) - 1 {
		copy(buffer.bytes[offset + 1:], buffer.bytes[offset:])
	}

	// Insert new char.
	buffer.bytes[offset] = u8(char)
	buffer.cursor.pos += 1
	buffer.dirty = true
	buffer_update_line_starts(buffer)
}

buffer_delete_char :: proc(buffer: ^Buffer) {
	if buffer.cursor.pos <= 0 do return

	// Remove character before cursor.
	if buffer.cursor.pos < len(buffer.bytes) {
		copy(buffer.bytes[buffer.cursor.pos - 1:], buffer.bytes[buffer.cursor.pos:])
	}
	resize(&buffer.bytes, len(buffer.bytes) - 1)

	buffer.cursor.pos -= 1
	buffer.dirty = true
	buffer_update_line_starts(buffer)
}

buffer_update_line_starts :: proc(buffer: ^Buffer) {
	// Clear existing line starts and add first line.
	clear(&buffer.line_starts)
	append(&buffer.line_starts, 0) // First line always start at 0.

	for i := 0; i < len(buffer.bytes); i += 1 {
		if buffer.bytes[i] == '\n' {
			append(&buffer.line_starts, i + 1) // First line always start at 0.
		}
	}

	// Update cursor line and column
	buffer.cursor.line = 0
	for i := 1; i < len(buffer.line_starts); i += 1 {
		if buffer.cursor.pos >= buffer.line_starts[i] {
			buffer.cursor.line = i
		}
	}
	buffer.cursor.col = buffer.cursor.pos - buffer.line_starts[buffer.cursor.line]
}

//
// Movement.
//

buffer_move_cursor :: proc(buffer: ^Buffer, movement: CursorMovement) {
	switch movement {
	case .Left:
		if buffer.cursor.pos > 0 do buffer.cursor.pos -= 1
	case .Right:
		if buffer.cursor.pos < len(buffer.bytes) do buffer.cursor.pos += 1
	case .Up:
		if buffer.cursor.line > 0 {
			// Get target column (preserved from current position).
			target_col := buffer.cursor.col
			new_line := buffer.cursor.line - 1 // Move to prev line.

			// Calculate new position.
			line_length := buffer_line_length(buffer, new_line)
			new_col := min(target_col, line_length)
			buffer.cursor.pos = buffer.line_starts[new_line] + new_col
		}
	case .Down:
		if buffer.cursor.line < len(buffer.line_starts) - 1 {
			// Same stuff as before.
			target_col := buffer.cursor.col
			new_line := buffer.cursor.line - 1

			// Calculate new position.
			line_length := buffer_line_length(buffer, new_line)
			new_col := min(target_col, line_length)
			buffer.cursor.pos = buffer.line_starts[new_line] + new_col
		}
	case .LineStart:
		buffer.cursor.pos = buffer.line_starts[buffer.cursor.line]
	case .LineEnd:
		current_line := buffer.cursor.line
		if current_line < len(buffer.line_starts) - 1 {
			buffer.cursor.pos = buffer.line_starts[current_line + 1] - 1
		} else {
			buffer.cursor.pos = len(buffer.bytes)
		}
	case .WordLeft:
		if buffer.cursor.pos > 0 { 	// Skip spaces backwards
			for buffer.cursor.pos > 0 &&
			    (buffer.bytes[buffer.cursor.pos - 1] == ' ' ||
					    buffer.bytes[buffer.cursor.pos - 1] == '\t' ||
					    buffer.bytes[buffer.cursor.pos - 1] == '\n') {
				buffer.cursor.pos -= 1
			}

			// Skip word backwards
			for buffer.cursor.pos > 0 &&
			    buffer.bytes[buffer.cursor.pos - 1] != ' ' &&
			    buffer.bytes[buffer.cursor.pos - 1] != '\t' &&
			    buffer.bytes[buffer.cursor.pos - 1] != '\n' {
				buffer.cursor.pos -= 1
			}
		}

	case .WordRight:
		if buffer.cursor.pos < len(buffer.bytes) {
			// Skip current word
			for buffer.cursor.pos < len(buffer.bytes) &&
			    buffer.bytes[buffer.cursor.pos] != ' ' &&
			    buffer.bytes[buffer.cursor.pos] != '\t' &&
			    buffer.bytes[buffer.cursor.pos] != '\n' {
				buffer.cursor.pos += 1
			}

			// Skip spaces
			for buffer.cursor.pos < len(buffer.bytes) &&
			    (buffer.bytes[buffer.cursor.pos] == ' ' ||
					    buffer.bytes[buffer.cursor.pos] == '\t' ||
					    buffer.bytes[buffer.cursor.pos] == '\n') {
				buffer.cursor.pos += 1
			}
		}
	}
}

//
// Drawing
//

buffer_draw_cursor :: proc(
	buffer: ^Buffer,
	position: rl.Vector2,
	font_size: f32,
	spacing: f32,
	font: rl.Font,
) {
	cursor_pos := position

	// Adjust vertical position based on line number.
	cursor_pos.y += f32(buffer.cursor.line) * (font_size + spacing)

	// Calculate horizontal position within the current line.
	if buffer.cursor.pos > 0 && len(buffer.bytes) > 0 {
		line_start := buffer.line_starts[buffer.cursor.line]
		line_text := buffer.bytes[line_start:buffer.cursor.pos]

		if len(line_text) > 0 {
			// Create temporary buffer for measurement.
			temp_text := make([dynamic]u8, len(line_text) + 1)
			defer delete(temp_text)

			copy(temp_text[:], line_text)
			temp_text[len(line_text)] = 0

			cursor_pos.x += rl.MeasureTextEx(font, cstring(&temp_text[0]), font_size, spacing).x
		}
	}

	// Blink effect
	if buffer.cursor.blink && (int(rl.GetTime() * 2) % 2 == 0) do return

	switch buffer.cursor.style {
	case .Bar:
		rl.DrawLineV(cursor_pos, {cursor_pos.x, cursor_pos.y + font_size}, buffer.cursor.color)
	case .Block:
		char_width := rl.MeasureTextEx(font, "@", font_size, spacing).x
		rl.DrawRectangleV(
			cursor_pos,
			{char_width, font_size},
			{buffer.cursor.color.r, buffer.cursor.color.g, buffer.cursor.color.b, 128},
		)
	case .Underscore:
		char_width := rl.MeasureTextEx(font, "M", font_size, spacing).x
		rl.DrawLineV(
			{cursor_pos.x, cursor_pos.y + font_size},
			{cursor_pos.x + char_width, cursor_pos.y + font_size},
			buffer.cursor.color,
		)
	}
}

buffer_draw :: proc(
	buffer: ^Buffer,
	position: rl.Vector2,
	font_size: f32,
	spacing: f32,
	font: rl.Font,
) {
	// Draw text only if we have some content in the buffer.
	if len(buffer.bytes) > 0 {
		// Ensure null termination for text display.
		append(&buffer.bytes, 0)
		defer resize(&buffer.bytes, len(buffer.bytes) - 1)

		// Draw main text.
		rl.DrawTextEx(font, cstring(&buffer.bytes[0]), position, font_size, spacing, rl.BLACK)
	}

	// Always draw the cursor, regardless of buffer content.
	buffer_draw_cursor(buffer, position, font_size, spacing, font)
}

//
// Helpers
//

// Returns the length of a specific line.
buffer_line_length :: proc(buffer: ^Buffer, line: int) -> int {
	if line >= len(buffer.line_starts) do return 0
	start := buffer.line_starts[line]
	end := len(buffer.bytes)

	if line < len(buffer.line_starts) - 1 {
		end = buffer.line_starts[line + 1]
	}

	return end - start
}
