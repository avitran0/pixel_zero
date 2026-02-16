use pixel_zero::{
    graphics::{Font, Frame},
    input::Input,
};

pub mod game_menu;
pub mod main_menu;

pub trait Screen {
    fn update(&mut self, input: &Input) -> Option<Box<dyn Screen>>;
    fn render(&self, frame: &mut Frame, font: &Font);
}
