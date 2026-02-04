use std::{
    fs::File,
    io::{BufWriter, Write as _},
    sync::atomic::{AtomicBool, Ordering},
};

use log::{Level, Log};
use nix::{
    errno::Errno,
    sys::termios::{FlushArg, LocalFlags, SetArg, Termios, tcflush, tcgetattr, tcsetattr},
};
use parking_lot::Mutex;

mod ffi;
pub mod graphics;
pub mod input;
mod io;

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

pub struct FileLogger {
    writer: Mutex<BufWriter<File>>,
    level: Level,
}

impl FileLogger {
    /// # Errors
    ///
    /// Might fail to open the log file.
    pub fn new(file_name: &str, level: Level) -> std::io::Result<Self> {
        let mut path = std::env::current_exe()?;
        path.pop();
        path.push(file_name);
        let file = File::options().create(true).append(true).open(path)?;

        Ok(Self {
            writer: Mutex::new(BufWriter::new(file)),
            level,
        })
    }

    /// # Panics
    ///
    /// Panics when another logger was already initialized.
    pub fn init(self) {
        let max_level = self.level.to_level_filter();
        log::set_boxed_logger(Box::new(self)).unwrap();
        log::set_max_level(max_level);
    }

    pub fn write_log(&self, record: &log::Record) {
        let message = format!("[{}] {}\n", record.level(), record.args());
        let mut writer = self.writer.lock();
        let _ = writer.write_all(message.as_bytes());
        let _ = writer.flush();
    }
}

impl Log for FileLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= self.level
    }

    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            self.write_log(record);
        }
    }

    fn flush(&self) {
        let _ = self.writer.lock().flush();
    }
}
