use std::sync::atomic::{AtomicBool, Ordering};

use nix::{
    errno::Errno,
    sys::termios::{FlushArg, LocalFlags, SetArg, Termios, tcflush, tcgetattr, tcsetattr},
};

/// This struct prevents keystrokes ending up in stdout while the program is running.
/// Only one should be created in `main`, and dropped on program exit.
pub struct TerminalGuard {
    original: Termios,
}

static CREATED: AtomicBool = AtomicBool::new(false);
impl TerminalGuard {
    /// # Errors
    ///
    /// Fails when a `TerminalGuard` is already in place.
    /// Can also fail `tcgetattr` or `tcsetattr` calls.
    pub fn new() -> Result<Self, Errno> {
        if CREATED.swap(true, Ordering::Relaxed) {
            return Err(Errno::EALREADY);
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
