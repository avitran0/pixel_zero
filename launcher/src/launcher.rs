use pixel_zero::{
    Frame, graphics::{Graphics, color::Color}, input::{Button, Input}
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
            let mut frame = Frame::default();
            frame.set_clear_color(Color::rgb(100, 150, 240));

            self.screen.render(&mut frame);

            self.graphics.present_frame(&frame).unwrap();
            self.graphics.check_error();
        }
    }
}
