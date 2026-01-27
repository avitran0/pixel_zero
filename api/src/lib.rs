use std::sync::atomic::{AtomicBool, Ordering};

use nix::sys::termios::{FlushArg, LocalFlags, SetArg, Termios, tcflush, tcgetattr, tcsetattr};

pub mod graphics;
pub mod input;

pub const WIDTH: u32 = 320;
pub const HEIGHT: u32 = 240;
pub const SOCKET_PATH: &str = "/tmp/pixel_zero.sock";

/// This struct prevents keystrokes ending up in stdout while the program is running.
/// Only one should be created in `main`, and dropped on program exit.
pub struct TerminalGuard {
    original: Termios,
}

static CREATED: AtomicBool = AtomicBool::new(false);
impl TerminalGuard {
    pub fn new() -> anyhow::Result<Self> {
        if CREATED.swap(true, Ordering::Relaxed) {
            return Err(anyhow::anyhow!("terminal guard already in use"));
        }

        let original = tcgetattr(std::io::stdin())?;
        let mut temporary = original.clone();
        temporary.local_flags &= !(LocalFlags::ICANON | LocalFlags::ECHO);
        tcsetattr(std::io::stdin(), SetArg::TCSANOW, &temporary)?;
        Ok(Self { original })
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        if tcflush(std::io::stdin(), FlushArg::TCIFLUSH).is_err() {
            log::error!("failed to flush stdin");
        }
        if tcsetattr(std::io::stdin(), SetArg::TCSANOW, &self.original).is_err() {
            log::error!("failed to reset termios");
        }
    }
}
