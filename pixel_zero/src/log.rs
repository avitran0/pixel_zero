use std::{
    fs::File,
    io::{BufWriter, Write as _},
};

use log::{Level, Log, SetLoggerError};
use parking_lot::Mutex;

/// Simple logger implementation that logs to a given file.
///
/// # Usage
///
/// ```
/// use pixel_zero::log::FileLogger;
///
/// FileLogger::install("filename.log", log::Level::Info);
/// ```
///
/// This will panic if it cannot open the given file or another logger is already installed.
/// Alternatively, to handle these errors, use this:
///
/// ```
/// use pixel_zero::log::FileLogger;
///
/// if let Ok(logger) = FileLogger::new("filename.log", log::Level::Info) {
///     let result = logger.init();
/// }
/// ```
pub struct FileLogger {
    writer: Mutex<BufWriter<File>>,
    level: Level,
}

impl FileLogger {
    /// # Panics
    ///
    /// Panics if it fails to open the given file,
    /// or when another logger is already installed.
    pub fn install(file_name: &str, level: Level) {
        Self::new(file_name, level).unwrap().init().unwrap();
    }

    /// # Errors
    ///
    /// Returns an `std::io::Error` if it fails to open the given file.
    pub fn new(file_name: &str, level: Level) -> std::io::Result<Self> {
        let mut path = std::env::current_exe()?;
        path.pop();
        path.push(file_name);
        let file = File::options()
            .create(true)
            .truncate(true)
            .write(true)
            .open(path)?;

        Ok(Self {
            writer: Mutex::new(BufWriter::new(file)),
            level,
        })
    }

    /// Installs the logger.
    ///
    /// # Errors
    ///
    /// Returns an `log::SetLoggerError` if another one has already been installed.
    pub fn init(self) -> Result<(), SetLoggerError> {
        let max_level = self.level.to_level_filter();
        log::set_boxed_logger(Box::new(self))?;
        log::set_max_level(max_level);
        Ok(())
    }

    fn write_log(&self, record: &log::Record) {
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
