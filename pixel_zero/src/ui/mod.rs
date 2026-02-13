use crate::graphics::Font;

pub struct Ui {
    id_stack: Vec<Id>,
}

impl Ui {
    pub fn label(&mut self, font: &Font, text: &str) {
        let text_size = font.text_size(text);
    }
}

pub struct Context {
    focus: Option<Id>,
    active: Option<Id>,
}

struct Id(u64);
