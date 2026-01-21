use ratatui::{
    buffer::Buffer,
    layout::Rect,
    widgets::{Block, Widget as _},
};

use crate::screen::Screen;

pub struct GameMenu {}

impl GameMenu {
    pub fn new() -> Self {
        Self {}
    }
}

impl Screen for GameMenu {
    fn render(&self, area: Rect, buf: &mut Buffer) {
        let block = Block::bordered().title("Games");
        block.render(area, buf);
    }

    fn handle_events(&mut self) {}
}
