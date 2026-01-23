use api::{graphics::GraphicsContext, input::Input};

pub mod game_menu;

pub trait Screen {
    fn update(&mut self, input: &Input);
    fn render(&self, graphics: &GraphicsContext);
}
