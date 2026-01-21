use ratatui::{DefaultTerminal, Frame};

use crate::screen::{Screen, game_menu::GameMenu};

pub struct Launcher {
    screen: Box<dyn Screen>,
    exit: bool,
}

impl Launcher {
    pub fn new() -> Self {
        Self {
            exit: false,
            screen: Box::new(GameMenu::new()),
        }
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> std::io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }

        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        todo!()
    }

    fn handle_events(&mut self) -> std::io::Result<()> {
        todo!()
    }
}
