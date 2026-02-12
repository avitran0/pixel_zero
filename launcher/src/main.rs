use pixel_zero::log::FileLogger;

use crate::launcher::Launcher;

mod launcher;
mod screen;

fn main() {
    FileLogger::install("launcher.log", log::Level::Info);
    Launcher::new().run();
}
