use std::{
    collections::HashMap,
    fs::File,
    io::Read,
    os::unix::fs::{FileTypeExt, OpenOptionsExt},
    time::{Duration, Instant},
};

use crate::input::keys::*;

mod keys;

#[repr(C)]
#[derive(Debug)]
struct InputEvent {
    time: nix::libc::timeval,
    kind: u16,
    code: u16,
    value: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ButtonState {
    Pressed,
    Released,
}

const SCAN_INTERVAL: Duration = Duration::from_secs(5);
pub struct Input {
    device_files: Vec<File>,
    last_scanned: Instant,
    current_state: HashMap<Button, ButtonState>,
    previous_state: HashMap<Button, ButtonState>,
}

impl Default for Input {
    fn default() -> Self {
        let device_files = Self::scan_devices();
        log::info!("found {} input devices", device_files.len());
        Self {
            device_files,
            last_scanned: Instant::now(),
            current_state: HashMap::new(),
            previous_state: HashMap::new(),
        }
    }
}

impl Input {
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

            let Ok(file) = File::options()
                .read(true)
                .custom_flags(nix::libc::O_NONBLOCK)
                .open(entry.path())
            else {
                continue;
            };
            devices.push(file);
        }
        devices
    }

    pub fn update(&mut self) {
        if self.last_scanned.elapsed() > SCAN_INTERVAL {
            self.device_files = Self::scan_devices();
            self.last_scanned = Instant::now();
        }

        self.previous_state = self.current_state.clone();

        for device in &mut self.device_files {
            let mut buf = [0u8; std::mem::size_of::<InputEvent>()];

            while matches!(device.read_exact(&mut buf), Ok(())) {
                let event = unsafe {
                    let ptr = buf.as_ptr().cast::<InputEvent>();
                    &*ptr
                };

                if event.kind != EV_KEY {
                    continue;
                }

                let button = match event.code {
                    KEY_UP | BTN_DPAD_UP => Button::Up,
                    KEY_DOWN | BTN_DPAD_DOWN => Button::Down,
                    KEY_LEFT | BTN_DPAD_LEFT => Button::Left,
                    KEY_RIGHT | BTN_DPAD_RIGHT => Button::Right,

                    KEY_A | BTN_SOUTH => Button::A,
                    KEY_B | BTN_EAST => Button::B,
                    KEY_DOT | BTN_START => Button::Start,
                    KEY_COMMA | BTN_SELECT => Button::Select,
                    KEY_L | BTN_TL => Button::L,
                    KEY_R | BTN_TR => Button::R,

                    KEY_ESC => std::process::exit(0),

                    _ => continue,
                };

                let state = if event.value == 0 {
                    ButtonState::Released
                } else {
                    ButtonState::Pressed
                };

                self.current_state.insert(button, state);
            }
        }
    }

    pub fn is_pressed(&self, button: Button) -> bool {
        self.current_state.get(&button) == Some(&ButtonState::Pressed)
    }

    pub fn just_pressed(&self, button: Button) -> bool {
        let current = self.current_state.get(&button) == Some(&ButtonState::Pressed);
        let previous = self.previous_state.get(&button) != Some(&ButtonState::Pressed);
        current && previous
    }

    pub fn just_released(&self, button: Button) -> bool {
        let current = self.current_state.get(&button) == Some(&ButtonState::Released);
        let previous = self.previous_state.get(&button) != Some(&ButtonState::Released);
        current && previous
    }
}
