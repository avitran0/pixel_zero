use api::{graphics::GraphicsContext, input::Input};

use crate::screen::Screen;

pub struct GameMenu {}

impl GameMenu {
    pub fn new() -> Self {
        Self {}
    }
}

impl Screen for GameMenu {
    fn update(&mut self, input: &Input) {}

    fn render(&self, graphics: &GraphicsContext) {}
}
