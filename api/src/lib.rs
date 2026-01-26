use nix::sys::termios::{LocalFlags, SetArg, Termios, tcgetattr, tcsetattr};

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
        if tcsetattr(std::io::stdin(), SetArg::TCSANOW, &self.original).is_err() {
            log::error!("failed to reset termios");
        }
    }
}
