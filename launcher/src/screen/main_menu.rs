use pixel_zero::{
    graphics::{Font, Frame},
    input::Input,
    ui::{Ui, UiFrame},
};

use crate::screen::Screen;

pub struct MainMenu {
    ui: Ui,
    ui_frame: UiFrame,

    test_bool: bool,
    test_int: f32,
}

impl MainMenu {
    pub fn init(font: &Font) -> Self {
        let ui = Ui::new(font.clone());
        let ui_frame = ui.start_frame();
        Self {
            ui,
            ui_frame,
            test_bool: false,
            test_int: 50.0,
        }
    }
}

impl Screen for MainMenu {
    fn update(&mut self, input: &Input) {
        self.ui_frame.button("Text Here");
        self.ui_frame.checkbox("test", &mut self.test_bool);
        self.ui_frame
            .slider("slider", &mut self.test_int, 0.0..=100.0);
    }

    fn render(&self, frame: &mut Frame, font: &Font) {
        self.ui_frame.render(frame);
    }
}
