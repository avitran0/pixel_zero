use std::{fs::File, io::Read, os::unix::fs::FileTypeExt};

const EV_KEY: u16 = 0x01;
const EV_ABS: u16 = 0x03;

const KEY_A: u16 = 30;
const KEY_B: u16 = 48;
const KEY_L: u16 = 38;
const KEY_R: u16 = 19;
// start
const KEY_DOT: u16 = 52;
// select
const KEY_COMMA: u16 = 51;

const KEY_UP: u16 = 103;
const KEY_DOWN: u16 = 108;
const KEY_LEFT: u16 = 105;
const KEY_RIGHT: u16 = 106;

const BTN_DPAD_UP: u16 = 0x220;
const BTN_DPAD_DOWN: u16 = 0x221;
const BTN_DPAD_LEFT: u16 = 0x222;
const BTN_DPAD_RIGHT: u16 = 0x223;

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

    fn read(&self) {
        for mut device in &self.device_files {
            let mut buf = vec![0u8; size_of::<InputEvent>()];
            while device.read_exact(&mut buf).is_ok() {
                let event = unsafe {
                    let ptr = buf.as_ptr() as *const InputEvent;
                    &*ptr
                };

                let kind = event.kind;
                if kind != EV_KEY && kind != EV_ABS {
                    continue;
                }

                let button = match (kind, event.code) {
                    (EV_KEY, KEY_UP) => Button::Up,
                    (EV_KEY, KEY_DOWN) => Button::Down,
                    (EV_KEY, KEY_LEFT) => Button::Left,
                    (EV_KEY, KEY_RIGHT) => Button::Right,

                    (EV_KEY, BTN_DPAD_UP) => Button::Up,
                    (EV_KEY, BTN_DPAD_DOWN) => Button::Down,
                    (EV_KEY, BTN_DPAD_LEFT) => Button::Left,
                    (EV_KEY, BTN_DPAD_RIGHT) => Button::Right,

                    (EV_KEY, KEY_A) => Button::A,
                    (EV_KEY, KEY_B) => Button::B,
                    (EV_KEY, KEY_L) => Button::L,
                    (EV_KEY, KEY_R) => Button::R,
                    (EV_KEY, KEY_DOT) => Button::Start,
                    (EV_KEY, KEY_COMMA) => Button::Select,

                    _ => continue,
                };
            }
        }
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
    L,
    R,
    Start,
    Select,
}

pub enum ButtonEvent {
    Pressed,
    Released,
}
