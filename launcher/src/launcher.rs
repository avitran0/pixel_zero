use std::time::{Duration, Instant};

use api::{
    graphics::GraphicsContext,
    input::{Button, Input},
};

use crate::screen::{Screen, game_menu::GameMenu};

const TIME: Duration = Duration::from_secs(5);

pub struct Launcher {
    graphics: GraphicsContext,
    input: Input,
    screen: Box<dyn Screen>,
    start: Instant,
    exit: bool,
}

impl Launcher {
    pub fn new() -> Self {
        Self {
            graphics: GraphicsContext::load().unwrap(),
            input: Input::default(),
            screen: Box::new(GameMenu::new()),
            start: Instant::now(),
            exit: false,
        }
    }

    pub fn run(&mut self) {
        while !self.exit || self.start.elapsed() > TIME {
            self.input.update();
            if self.input.just_pressed(Button::A) {
                self.exit = true;
            }
            self.screen.update(&self.input);
            self.graphics.clear();
            self.screen.render(&self.graphics);
            self.graphics.present().unwrap();
        }
    }
}
