// This file is compiled as part of the `odin.dll` file. It contains the
// procs that `atlas_hot_reload.exe` will call, such as:
//
// atlas_init: Sets up the atlas state
// atlas_update: Run once per frame
// atlas_shutdown: Shuts down atlas and frees memory
// atlas_memory: Run just before a hot reload, so atlas.exe has a pointer to the
//		atlas's memory.
// atlas_hot_reloaded: Run after a hot reload so that the `g_mem` global variable
//		can be set to whatever pointer it was in the old DLL.
//
// Note: When compiled as part of the release executable this whole package is imported as a normal
// odin package instead of a DLL.

package atlas

import eg "engine"
import rl "vendor:raylib"

AtlasState :: struct {
	buffer:    eg.Buffer,
	font:      rl.Font,
	font_size: f32,
	spacing:   f32,
}

atlas_mem: ^AtlasState

// This is where we update the state based on user interactions.
update :: proc() {
	key := rl.GetCharPressed()

	for key != 0 {
		if key >= 32 && key < 127 {
			eg.buffer_insert_char(&atlas_mem.buffer, rune(key))
		}

		key = rl.GetCharPressed()
	}

	if rl.IsKeyPressed(rl.KeyboardKey.BACKSPACE) ||
	   rl.IsKeyPressedRepeat(rl.KeyboardKey.BACKSPACE) {
		eg.buffer_delete_char(&atlas_mem.buffer)
	}

	// Cursor movement.
	if rl.IsKeyPressed(.LEFT) || rl.IsKeyPressedRepeat(rl.KeyboardKey.LEFT) do eg.buffer_move_cursor(&atlas_mem.buffer, .Left)
	if rl.IsKeyPressed(.RIGHT) || rl.IsKeyPressedRepeat(rl.KeyboardKey.RIGHT) do eg.buffer_move_cursor(&atlas_mem.buffer, .Right)
	if rl.IsKeyPressed(.UP) || rl.IsKeyPressedRepeat(rl.KeyboardKey.UP) do eg.buffer_move_cursor(&atlas_mem.buffer, .Up)
	if rl.IsKeyPressed(.DOWN) || rl.IsKeyPressedRepeat(rl.KeyboardKey.DOWN) do eg.buffer_move_cursor(&atlas_mem.buffer, .Down)
	if rl.IsKeyPressed(.HOME) do eg.buffer_move_cursor(&atlas_mem.buffer, .LineStart)
	if rl.IsKeyPressed(.END) do eg.buffer_move_cursor(&atlas_mem.buffer, .LineEnd)

	// Enter key.
	if rl.IsKeyPressed(.ENTER) do eg.buffer_insert_char(&atlas_mem.buffer, '\n')

	// Word movement (with Ctrl key).
	if rl.IsKeyDown(.LEFT_CONTROL) {
		if rl.IsKeyPressed(.LEFT) {
			eg.buffer_move_cursor(&atlas_mem.buffer, .WordLeft)
		}
		if rl.IsKeyPressed(.RIGHT) {
			eg.buffer_move_cursor(&atlas_mem.buffer, .WordRight)
		}
	}
}

// Draws the state.
// Note: main_hot_reload.odin clears the temp allocator at end of frame.
draw :: proc() {
	rl.BeginDrawing()
	rl.ClearBackground(rl.GRAY)
	eg.buffer_draw(
		&atlas_mem.buffer,
		rl.Vector2{10, 10},
		atlas_mem.font_size,
		atlas_mem.spacing,
		atlas_mem.font,
	)

	rl.EndDrawing()
}

@(export)
atlas_update :: proc() -> bool {
	update()
	draw()
	return !rl.WindowShouldClose()
}

@(export)
atlas_init_window :: proc() {
	rl.SetConfigFlags({.WINDOW_RESIZABLE, .VSYNC_HINT})
	rl.InitWindow(1280, 720, "Atlas text editor")
	rl.SetTargetFPS(500)
}

// Initialize the main state of Atlas.
// This also initializes the font.
@(export)
atlas_init :: proc() {
	atlas_mem = new(AtlasState)
	// TODO: Treat this I/O a little better.
	atlas_mem.font = rl.LoadFont("./fonts/iosevka-regular.ttf")
	atlas_mem.font_size = 20.0
	atlas_mem.spacing = 2.0

	// Initialize buffer with the loaded font.
	atlas_mem.buffer = eg.buffer_init(&atlas_mem.font)

	atlas_hot_reloaded(atlas_mem)
}

@(export)
atlas_shutdown :: proc() {
	if atlas_mem != nil {
		eg.buffer_free(&atlas_mem.buffer)
		rl.UnloadFont(atlas_mem.font)
		free(atlas_mem)
		atlas_mem = nil
	}
}

@(export)
atlas_shutdown_window :: proc() {
	rl.CloseWindow()
}

@(export)
atlas_memory :: proc() -> rawptr {
	return atlas_mem
}

@(export)
atlas_memory_size :: proc() -> int {
	return size_of(AtlasState)
}

@(export)
atlas_hot_reloaded :: proc(mem: rawptr) {
	atlas_mem = (^AtlasState)(mem)
}

@(export)
atlas_force_reload :: proc() -> bool {
	return rl.IsKeyPressed(.F5)
}

@(export)
atlas_force_restart :: proc() -> bool {
	return rl.IsKeyPressed(.F6)
}
