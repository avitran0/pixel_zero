use std::time::{Duration, Instant};

use glam::{Vec2, uvec2};
use pixel_zero::{
    graphics::{Graphics, color::Color, sprite::Sprite},
    input::{Button, Input},
};

use crate::screen::{Screen, game_menu::GameMenu};

const TIME: Duration = Duration::from_secs(5);

pub struct Launcher {
    graphics: Graphics,
    input: Input,
    screen: Box<dyn Screen>,
    start: Instant,
    exit: bool,
}

impl Launcher {
    pub fn new() -> Self {
        Self {
            graphics: Graphics::load().unwrap(),
            input: Input::default(),
            screen: Box::new(GameMenu::new()),
            start: Instant::now(),
            exit: false,
        }
    }

    pub fn run(&mut self) {
        let sprite = Sprite::load_binary(include_bytes!("redstone.png")).unwrap();
        let mut position = Vec2::new(0.0, 0.0);
        const SPEED: f32 = 0.1;

        while !self.exit && self.start.elapsed() < TIME {
            self.input.update();
            if self.input.just_pressed(Button::A) {
                println!("exiting");
                self.exit = true;
            }
            if self.input.is_pressed(Button::Left) {
                position.x -= SPEED;
            }
            if self.input.is_pressed(Button::Right) {
                position.x += SPEED;
            }
            if self.input.is_pressed(Button::Up) {
                position.y -= SPEED;
            }
            if self.input.is_pressed(Button::Down) {
                position.y += SPEED;
            }
            self.screen.update(&self.input);
            self.graphics.clear(Color::rgb(100, 150, 240));

            self.graphics
                .draw_sprite(&sprite, uvec2(position.x as u32, position.y as u32));
            self.screen.render(&self.graphics);

            self.graphics.present().unwrap();
            self.graphics.check_error();
        }
    }
}
