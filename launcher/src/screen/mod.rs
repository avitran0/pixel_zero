use api::{graphics::Graphics, input::Input};

pub mod game_menu;

pub trait Screen {
    fn update(&mut self, input: &Input);
    fn render(&self, graphics: &Graphics);
}
