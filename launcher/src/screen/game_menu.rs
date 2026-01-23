use api::{
    graphics::{GraphicsContext, color::Color, sprite::Sprite, font::BitmapFont},
    input::Input,
};

use crate::screen::Screen;

pub struct GameMenu {
    test_sprite: Sprite,
    font: BitmapFont,
}

impl GameMenu {
    pub fn new() -> Self {
        // Create a simple test sprite (16x16 red square with white border)
        let mut test_sprite = Sprite::new(16, 16);
        for y in 0..16 {
            for x in 0..16 {
                if x == 0 || x == 15 || y == 0 || y == 15 {
                    test_sprite.set_pixel(x, y, Color::WHITE);
                } else {
                    test_sprite.set_pixel(x, y, Color::RED);
                }
            }
        }

        Self {
            test_sprite,
            font: BitmapFont::default(),
        }
    }
}

impl Screen for GameMenu {
    fn update(&mut self, input: &Input) {}

    fn render(&self, graphics: &mut GraphicsContext) {
        // Clear the framebuffer
        graphics.clear_framebuffer(Color::rgb(50, 50, 100));

        // Draw test sprites at various positions
        graphics.draw_sprite(&self.test_sprite, 10, 10);
        graphics.draw_sprite(&self.test_sprite, 50, 50);
        graphics.draw_sprite(&self.test_sprite, 150, 100);

        // Draw some text
        graphics.draw_text(&self.font, "Hello, Pixel Zero!", 10, 150, Color::WHITE);
        graphics.draw_text(&self.font, "Press A to exit", 10, 170, Color::GREEN);
    }
}
