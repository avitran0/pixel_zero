use pixel_zero::{
    graphics::{Font, Frame, Graphics, color::Color},
    input::{Button, Input},
};

use crate::screen::{Screen, game_menu::GameMenu, main_menu::MainMenu};

pub struct Launcher {
    graphics: Graphics,
    input: Input,
    font: Font,
    screen: Box<dyn Screen>,
    exit: bool,
}

impl Launcher {
    pub fn new() -> Self {
        let graphics = Graphics::load().unwrap();
        let font = graphics
            .load_font_binary(include_bytes!("../assets/cozette.psf"))
            .unwrap();
        let screen = Box::new(MainMenu::init(&font));

        Self {
            graphics,
            input: Input::default(),
            font,
            screen,
            exit: false,
        }
    }

    pub fn run(&mut self) {
        while !self.exit {
            self.input.update();
            if self.input.just_pressed(Button::Start) {
                self.exit = true;
            }
            self.screen.update(&self.input);
            let mut frame = Frame::default();
            frame.set_clear_color(Color::rgb(100, 150, 240));

            self.screen.render(&mut frame, &self.font);

            self.graphics.present_frame(&frame).unwrap();
        }
    }
}
