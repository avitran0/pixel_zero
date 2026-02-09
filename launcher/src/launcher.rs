use pixel_zero::{
    graphics::{Graphics, color::Color},
    input::{Button, Input},
};

use crate::screen::{Screen, game_menu::GameMenu};

pub struct Launcher {
    graphics: Graphics,
    input: Input,
    screen: Box<dyn Screen>,
    exit: bool,
}

impl Launcher {
    pub fn new() -> Self {
        Self {
            graphics: Graphics::load().unwrap(),
            input: Input::default(),
            screen: Box::new(GameMenu::new()),
            exit: false,
        }
    }

    pub fn run(&mut self) {
        while !self.exit {
            self.input.update();
            if self.input.just_pressed(Button::A) {
                log::info!("exiting");
                self.exit = true;
            }
            self.screen.update(&self.input);
            self.graphics.clear(Color::rgb(100, 150, 240));

            self.screen.render(&self.graphics);

            self.graphics.present().unwrap();
            self.graphics.check_error();
        }
    }
}
