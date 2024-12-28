package main

import rl "vendor:raylib"

screen_width := rl.GetScreenWidth()
screen_height := rl.GetScreenHeight()
text_buffer: [1024]u8
text_len := 0
font := rl.GetFontDefault()
font_size: f32 = 20.0
spacing: f32 = 2.0

main :: proc() {

	rl.SetConfigFlags(rl.ConfigFlags{.WINDOW_RESIZABLE, .WINDOW_MAXIMIZED})
	rl.InitWindow(screen_width, screen_height, "atlas editor")
	rl.SetTargetFPS(60)
	defer rl.CloseWindow()

	for !rl.WindowShouldClose() {
		key := rl.GetCharPressed()
		for key != 0 {
			if key >= 32 && key < 127 && text_len < len(text_buffer) - 1 {
				text_buffer[text_len] = cast(u8)key
				text_len += 1

				text_buffer[text_len] = 0
			}

			key = rl.GetCharPressed()
		}

		if rl.IsKeyPressed(rl.KeyboardKey.BACKSPACE) {
			if text_len > 0 {
				text_len -= 1
				text_buffer[text_len] = 0
			}
		}

		rl.BeginDrawing()
		defer rl.EndDrawing()
		rl.ClearBackground(rl.WHITE)

		rl.DrawTextEx(
			font,
			cast(cstring)&text_buffer[0], 
			rl.Vector2{10, 10},
			font_size,
			spacing,
			rl.BLACK,
		)
	}
}

