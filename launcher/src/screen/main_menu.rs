use pixel_zero::{
    graphics::{Font, Frame},
    input::Input,
    ui::Ui,
};

use crate::screen::{Screen, game_menu::GameMenu};

pub struct MainMenu {
    ui: Ui,

    test_bool: bool,
    test_int: i32,
}

impl MainMenu {
    pub fn init(font: &Font) -> Self {
        let ui = Ui::new(font.clone());
        Self {
            ui,
            test_bool: false,
            test_int: 50,
        }
    }
}

impl Screen for MainMenu {
    fn update(&mut self, input: &Input) -> Option<Box<dyn Screen>> {
        self.ui.update_input(input);
        self.ui.begin_frame();
        if self.ui.button("Text Here") {
            return Some(Box::new(GameMenu::init(self.ui.font())))
        }
        self.ui.checkbox("test", &mut self.test_bool);
        self.ui.slider("slider", &mut self.test_int, 0..=100);
        self.ui.label(&format!("{}", self.test_int));

        None
    }

    fn render(&self, frame: &mut Frame) {
        self.ui.render(frame);
    }
}
