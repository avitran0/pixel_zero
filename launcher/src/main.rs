use pixel_zero::{log::FileLogger, terminal::TerminalGuard};

use crate::launcher::Launcher;

mod launcher;
mod screen;

fn main() {
    FileLogger::new("launcher.log", log::Level::Info)
        .unwrap()
        .init();

    let _guard = TerminalGuard::new().unwrap();
    Launcher::new().run();
}
