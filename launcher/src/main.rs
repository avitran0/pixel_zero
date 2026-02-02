use pixel_zero::TerminalGuard;

use crate::launcher::Launcher;
use std::io::Write as _;

mod launcher;
mod screen;

fn main() {
    env_logger::builder()
        .format(|buf, record| writeln!(buf, "[{}] {}", record.level(), record.args()))
        .filter_level(log::LevelFilter::Info)
        .target(env_logger::Target::Stdout)
        .init();

    let _guard = TerminalGuard::new().unwrap();
    Launcher::new().run();
}
