package main

import "core:unicode/utf8"
import eg "engine"
import rl "vendor:raylib"

screen_width := rl.GetScreenWidth()
screen_height := rl.GetScreenHeight()
font_size: f32 = 20.0
spacing: f32 = 2.0

main :: proc() {
	rl.SetConfigFlags(rl.ConfigFlags{.WINDOW_RESIZABLE, .WINDOW_MAXIMIZED})
	rl.InitWindow(screen_width, screen_height, "atlas editor")
	rl.SetTargetFPS(60)
	defer rl.CloseWindow()

	font := rl.LoadFont("./fonts/iosevka-regular.ttf")
	defer rl.UnloadFont(font)

	buffer := eg.buffer_init(&font)
	defer eg.buffer_free(&buffer)

	buffer.cursor.style = .Bar
	buffer.cursor.color = rl.RED
	buffer.cursor.blink = false

	for !rl.WindowShouldClose() {
		key := rl.GetCharPressed()
		for key != 0 {
			if key >= 32 && key < 127 {
				eg.buffer_insert_char(&buffer, rune(key))
			}

			key = rl.GetCharPressed()
		}

		if rl.IsKeyPressed(rl.KeyboardKey.BACKSPACE) {
			eg.buffer_delete_char(&buffer)
		}

		rl.BeginDrawing()
		defer rl.EndDrawing()
		rl.ClearBackground(rl.WHITE)

		eg.buffer_draw(&buffer, rl.Vector2{10, 10}, font_size, spacing)
	}
}
