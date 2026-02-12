use pixel_zero::{
    Font, Frame,
    graphics::{Graphics, color::Color},
    input::{Button, Input},
};

use crate::screen::{Screen, game_menu::GameMenu};

pub struct Launcher {
    graphics: Graphics,
    input: Input,
    font: Font,
    screen: Box<dyn Screen>,
    exit: bool,
}

impl Launcher {
    pub fn new() -> Self {
        Self {
            graphics: Graphics::load().unwrap(),
            input: Input::default(),
            font: Font::load_binary(include_bytes!("../assets/cozette.psf")).unwrap(),
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

            self.screen.render(&mut frame, &self.font);

            self.graphics.present_frame(&frame).unwrap();
            self.graphics.check_error();
        }
    }
}
