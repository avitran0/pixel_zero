use std::{fs::File, os::unix::fs::FileTypeExt};

pub struct Input {}

impl Input {
    fn scan_devices() -> Vec<File> {
        let mut devices = Vec::new();
        for entry in std::fs::read_dir("/dev/input").unwrap() {
            let Ok(entry) = entry else {
                continue;
            };

            let Ok(file_type) = entry.file_type() else {continue;};
            if !file_type.is_char_device() {
                continue;
            }

            let file_name = entry.file_name();
            let Some(file_name) = file_name.to_str() else {continue;};

            if !file_name.starts_with("event") {
                continue;
            }

            let 
        }
        devices
    }
}
