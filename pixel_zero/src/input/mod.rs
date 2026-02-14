use std::{
    fs::File,
    io::Read,
    os::{
        fd::AsRawFd,
        unix::fs::{FileTypeExt, OpenOptionsExt},
    },
    time::{Duration, Instant},
};

use nix::ioctl_read_buf;
use strum::EnumCount;

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

/// Button layout similar to a Gameboy Advance.
#[derive(Debug, Clone, Copy, EnumCount)]
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

impl Button {
    /// Returns the discriminant of the variant, for usage in the state array.
    pub fn index(&self) -> usize {
        *self as usize
    }

    /// Tries to construct a `Button` from an index.
    pub fn from_usize(button: usize) -> Option<Self> {
        Some(match button {
            0 => Self::Up,
            1 => Self::Down,
            2 => Self::Left,
            3 => Self::Right,
            4 => Self::A,
            5 => Self::B,
            6 => Self::L,
            7 => Self::R,
            8 => Self::Start,
            9 => Self::Select,
            _ => return None,
        })
    }

    pub const BUTTON_COUNT: usize = Self::COUNT;
}

const SCAN_INTERVAL: Duration = Duration::from_secs(5);

pub struct Input {
    device_files: Vec<File>,
    last_scanned: Instant,
    current_state: [bool; Button::COUNT],
    previous_state: [bool; Button::COUNT],
}

impl Default for Input {
    fn default() -> Self {
        let device_files = Self::scan_devices();
        log::info!("found {} input devices", device_files.len());
        Self {
            device_files,
            last_scanned: Instant::now(),
            current_state: [false; Button::COUNT],
            previous_state: [false; Button::COUNT],
        }
    }
}

ioctl_read_buf!(key_bits, b'E', 0x20 + EV_KEY, u8);

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

            let mut bits = [0u8; 1024];
            if unsafe { key_bits(file.as_raw_fd(), &mut bits) }.is_err() {
                continue;
            }

            if !Self::has_bit(&bits, KEY_ESC) && !Self::has_bit(&bits, BTN_START) {
                continue;
            }

            devices.push(file);
        }
        devices
    }

    fn has_bit(bits: &[u8], bit: u16) -> bool {
        let byte = bits[(bit / 8) as usize];
        let mask = 1 << (bit % 8);
        byte & mask != 0
    }

    /// Updates the input state.
    /// Should be called once per game loop iteration, usually at the start.
    pub fn update(&mut self) {
        if self.last_scanned.elapsed() > SCAN_INTERVAL {
            self.device_files = Self::scan_devices();
            self.last_scanned = Instant::now();
        }

        self.previous_state = self.current_state;

        for device in &mut self.device_files {
            let mut buf = [0u8; std::mem::size_of::<InputEvent>()];

            while matches!(device.read_exact(&mut buf), Ok(())) {
                let event = unsafe {
                    let ptr = buf.as_ptr().cast::<InputEvent>();
                    &*ptr
                };

                match event.kind {
                    EV_KEY => {
                        if let Some((button, state)) = Self::handle_key_event(event) {
                            self.current_state[button.index()] = state;
                        }
                    }
                    EV_ABS => {
                        if let Some(axis_value) = Self::handle_abs_event(event) {
                            axis_value.apply(&mut self.current_state);
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    fn handle_key_event(event: &InputEvent) -> Option<(Button, bool)> {
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

            _ => return None,
        };

        let state = event.value != 0;

        Some((button, state))
    }

    fn handle_abs_event(event: &InputEvent) -> Option<AxisValue> {
        Some(match event.code {
            ABS_X => AxisValue {
                axis: Axis::X,
                value: event.value,
            },
            ABS_Y => AxisValue {
                axis: Axis::Y,
                value: event.value,
            },
            ABS_HAT0X => AxisValue {
                axis: Axis::X,
                value: event.value * THRESHOLD,
            },
            ABS_HAT0Y => AxisValue {
                axis: Axis::Y,
                value: event.value * THRESHOLD,
            },
            _ => return None,
        })
    }

    /// Whether a `Button` is pressed.
    pub fn is_pressed(&self, button: Button) -> bool {
        self.current_state[button.index()]
    }

    /// Whether a `Button` was just pressed.
    pub fn just_pressed(&self, button: Button) -> bool {
        let current = self.current_state[button.index()];
        let previous = self.previous_state[button.index()];
        current && !previous
    }

    /// Whether a `Button` was just released.
    pub fn just_released(&self, button: Button) -> bool {
        let current = self.current_state[button.index()];
        let previous = self.previous_state[button.index()];
        !current && previous
    }

    /// Returns the internal button state.
    /// `Button::index()` returns the index for this array.
    pub fn state(&self) -> &[bool; Button::COUNT] {
        &self.current_state
    }
}

#[derive(Debug)]
struct AxisValue {
    axis: Axis,
    value: i32,
}

impl AxisValue {
    fn apply(&self, button_state: &mut [bool; Button::COUNT]) {
        let (negative, positive) = self.axis.buttons();
        button_state[negative.index()] = self.value <= -DEADZONE;
        button_state[positive.index()] = self.value >= DEADZONE;
    }
}

#[derive(Debug)]
enum Axis {
    X,
    Y,
}

impl Axis {
    fn buttons(&self) -> (Button, Button) {
        match self {
            Axis::X => (Button::Left, Button::Right),
            Axis::Y => (Button::Up, Button::Down),
        }
    }
}
