use crate::launcher::Launcher;

mod launcher;
mod screen;

fn main() {
    Launcher::new().run();
}
