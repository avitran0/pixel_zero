use pixel_zero::{
    graphics::{Font, Frame},
    input::Input,
    ui::Ui,
};

use crate::screen::Screen;

pub struct MainMenu {
    ui: Ui,
}

impl MainMenu {
    pub fn init(font: &Font) -> Self {
        Self {
            ui: Ui::new(font.clone()),
        }
    }
}

impl Screen for MainMenu {
    fn update(&mut self, input: &Input) {}

    fn render(&self, frame: &mut Frame, font: &Font) {
        let mut ui_frame = self.ui.start_frame();

        ui_frame.button("Text Here");

        ui_frame.render(frame);
    }
}
