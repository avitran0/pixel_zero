use crate::launcher::Launcher;

mod launcher;
mod screen;

fn main() {
    ratatui::run(|terminal| Launcher::new().run(terminal)).unwrap();
}
