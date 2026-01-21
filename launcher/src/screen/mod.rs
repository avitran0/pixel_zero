use ratatui::{buffer::Buffer, layout::Rect};

pub mod game_menu;

pub trait Screen {
    fn render(&self, area: Rect, buf: &mut Buffer);
    fn handle_events(&mut self);
}
