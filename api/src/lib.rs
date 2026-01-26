use nix::sys::termios::{FlushArg, LocalFlags, SetArg, Termios, tcflush, tcgetattr, tcsetattr};

pub mod graphics;
pub mod input;

pub struct TerminalGuard {
    original: Termios,
}

impl TerminalGuard {
    pub fn new() -> anyhow::Result<Self> {
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
