use pixel_zero::{Font, Frame, input::Input};

pub mod game_menu;

pub trait Screen {
    fn update(&mut self, input: &Input);
    fn render(&self, frame: &mut Frame, font: &Font);
}
