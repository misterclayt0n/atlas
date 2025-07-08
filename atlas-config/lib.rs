// NOTE: Yes, for now this is very much useless.
// But will come in handy in the future when we want to set colorschemes and other various configurations
// en editor might want to hold.
// For now, we just store a simple font size constant really.

use iced::Pixels;

const DEFAULT_FONT_SIZE: f32 = 50.0;

#[derive(Clone, Copy)]
pub struct Config {
    pub font_size: Pixels
}

impl Default for Config {
    fn default() -> Self {
        Self {
            font_size: Pixels(DEFAULT_FONT_SIZE)
        }
    }
}
