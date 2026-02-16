use pixel_zero::{
    graphics::{Font, Frame},
    input::Input,
    ui::Ui,
};

use crate::screen::Screen;

pub struct MainMenu {
    ui: Ui,

    test_bool: bool,
    test_int: f32,
}

impl MainMenu {
    pub fn init(font: &Font) -> Self {
        let ui = Ui::new(font.clone());
        Self {
            ui,
            test_bool: false,
            test_int: 50.0,
        }
    }
}

impl Screen for MainMenu {
    fn update(&mut self, input: &Input) {
        self.ui.update_input(input);
        self.ui.begin_frame();
        self.ui.button("Text Here");
        self.ui.checkbox("test", &mut self.test_bool);
        self.ui.slider("slider", &mut self.test_int, 0.0..=100.0);
    }

    fn render(&self, frame: &mut Frame, font: &Font) {
        self.ui.render(frame);
    }
}
