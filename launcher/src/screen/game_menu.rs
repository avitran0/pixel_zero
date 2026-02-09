use glam::{Vec2, ivec2};
use pixel_zero::{
    graphics::{Graphics, sprite::Sprite},
    input::{Button, Input},
};

use crate::screen::Screen;

pub struct GameMenu {
    sprite: Sprite,
    position: Vec2,
}

impl GameMenu {
    pub fn new() -> Self {
        Self {
            sprite: Sprite::load_binary(include_bytes!("redstone.png")).unwrap(),
            position: Vec2::ZERO,
        }
    }

    const SPEED: f32 = 0.1;
}

impl Screen for GameMenu {
    fn update(&mut self, input: &Input) {
        if input.is_pressed(Button::Left) {
            self.position.x -= Self::SPEED;
        }
        if input.is_pressed(Button::Right) {
            self.position.x += Self::SPEED;
        }
        if input.is_pressed(Button::Up) {
            self.position.y -= Self::SPEED;
        }
        if input.is_pressed(Button::Down) {
            self.position.y += Self::SPEED;
        }
    }

    fn render(&self, graphics: &Graphics) {
        graphics.draw_sprite(
            &self.sprite,
            ivec2(self.position.x as i32, self.position.y as i32),
        );
    }
}
