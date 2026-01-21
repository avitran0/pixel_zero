use std::{fs::File, os::unix::fs::FileTypeExt};

pub struct Input {
    device_files: Vec<File>,
}

impl Input {
    pub fn new() -> anyhow::Result<Self> {
        let device_files = Self::scan_devices();
        Ok(Self { device_files })
    }

    fn scan_devices() -> Vec<File> {
        let mut devices = Vec::new();
        for entry in std::fs::read_dir("/dev/input").unwrap() {
            let Ok(entry) = entry else {
                continue;
            };

            let Ok(file_type) = entry.file_type() else {
                continue;
            };
            if !file_type.is_char_device() {
                continue;
            }

            let file_name = entry.file_name();
            let Some(file_name) = file_name.to_str() else {
                continue;
            };

            if !file_name.starts_with("event") {
                continue;
            }

            let Ok(file) = File::open(entry.path()) else {
                continue;
            };
            devices.push(file);
        }
        devices
    }
}

#[repr(C)]
#[derive(Debug)]
struct InputEvent {
    time: libc::timeval,
    kind: u16,
    code: u16,
    value: i32,
}

#[derive(Debug, Clone, Copy)]
pub enum Button {
    Up,
    Down,
    Left,
    Right,
    A,
    B,
    Start,
    Select,
}


