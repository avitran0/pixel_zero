use std::{
    collections::HashMap,
    fs::File,
    io::Read,
    os::unix::fs::{FileTypeExt, OpenOptionsExt},
    time::{Duration, Instant},
};

const EV_KEY: u16 = 0x01;

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

const BTN_SOUTH: u16 = 0x130;
const BTN_EAST: u16 = 0x131;
const BTN_START: u16 = 0x134;
const BTN_SELECT: u16 = 0x136;
const BTN_TL: u16 = 0x137;
const BTN_TR: u16 = 0x138;

#[repr(C)]
#[derive(Debug)]
struct InputEvent {
    time: libc::timeval,
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
                .custom_flags(libc::O_NONBLOCK)
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

            while let Ok(()) = device.read_exact(&mut buf) {
                let event = unsafe {
                    let ptr = buf.as_ptr() as *const InputEvent;
                    &*ptr
                };

                if event.kind != EV_KEY {
                    continue;
                }

                let button = match event.code {
                    KEY_UP => Button::Up,
                    KEY_DOWN => Button::Down,
                    KEY_LEFT => Button::Left,
                    KEY_RIGHT => Button::Right,

                    KEY_A => Button::A,
                    KEY_B => Button::B,
                    KEY_L => Button::L,
                    KEY_R => Button::R,
                    KEY_DOT => Button::Start,
                    KEY_COMMA => Button::Select,

                    BTN_DPAD_UP => Button::Up,
                    BTN_DPAD_DOWN => Button::Down,
                    BTN_DPAD_LEFT => Button::Left,
                    BTN_DPAD_RIGHT => Button::Right,

                    BTN_SOUTH => Button::A,
                    BTN_EAST => Button::B,
                    BTN_START => Button::Start,
                    BTN_SELECT => Button::Select,
                    BTN_TL => Button::L,
                    BTN_TR => Button::R,

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
