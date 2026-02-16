use pixel_zero::{
    graphics::{Font, Frame, Graphics, color::Color},
    input::{Button, Input},
};

use crate::screen::{Screen, main_menu::MainMenu};

pub struct Launcher {
    graphics: Graphics,
    input: Input,
    _font: Font,
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
            _font: font,
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
            let screen = self.screen.update(&self.input);

            let mut frame = Frame::default();
            frame.set_clear_color(Color::rgb(100, 150, 240));

            self.screen.render(&mut frame);

            if let Some(screen) = screen {
                self.screen = screen;
            }

            self.graphics.present_frame(&frame).unwrap();
        }
    }
}
