use pixel_zero::{log::FileLogger, terminal::TerminalGuard};

use crate::launcher::Launcher;

mod launcher;
mod screen;

fn main() {
    FileLogger::install("launcher.log", log::Level::Info);

    let _guard = TerminalGuard::new().unwrap();
    Launcher::new().run();
}
