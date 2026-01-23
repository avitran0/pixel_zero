use api::{graphics::GraphicsContext, input::Input};

use crate::screen::{Screen, game_menu::GameMenu};

pub struct Launcher {
    graphics: GraphicsContext,
    input: Input,
    screen: Box<dyn Screen>,
    exit: bool,
}

impl Launcher {
    pub fn new() -> Self {
        Self {
            graphics: GraphicsContext::load().unwrap(),
            input: Input::default(),
            screen: Box::new(GameMenu::new()),
            exit: false,
        }
    }

    pub fn run(&mut self) {
        while !self.exit {
            self.screen.update(&self.input);
            self.graphics.clear();
            self.screen.render(&self.graphics);
            self.graphics.present().unwrap();
        }
    }
}
