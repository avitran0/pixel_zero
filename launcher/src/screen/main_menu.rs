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

        self.ui.set_layout_width(180);
        if self.ui.button("Start Game") {
            return Some(Box::new(GameMenu::init(self.ui.font())));
        }
        self.ui.checkbox("Show FPS", &mut self.test_bool);
        self.ui.slider("Volume", &mut self.test_int, 0..=100);
        self.ui.progress_bar(self.test_int, 0..=100);
        self.ui.label(&format!("Volume: {}", self.test_int));

        self.ui.separator();
        self.ui.begin_columns(2);
        self.ui.label("Left Column");
        self.ui.button("Alpha");
        self.ui.button("Beta");
        self.ui.next_column();
        self.ui.label("Right Column");
        self.ui.button("Gamma");
        self.ui.button("Delta");
        self.ui.end_columns();

        None
    }

    fn render(&self, frame: &mut Frame) {
        self.ui.render(frame);
    }
}
