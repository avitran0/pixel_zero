use pixel_zero::{FileLogger, TerminalGuard};

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
