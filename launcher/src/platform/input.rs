use std::fs::File;

pub struct Input {}

impl Input {
    fn scan_devices() -> Vec<File> {
        for entry in std::fs::read_dir("/dev/input").unwrap() {
            let Ok(entry) = entry else {
                continue;
            };
        }
        vec![]
    }
}
