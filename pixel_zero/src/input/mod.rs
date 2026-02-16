use std::{
    fs::File,
    os::{
        fd::AsRawFd,
        unix::fs::{FileTypeExt, OpenOptionsExt},
    },
    time::{Duration, Instant},
};

use nix::{ioctl_read, ioctl_read_buf};
use strum::{EnumCount, EnumIter};

use crate::input::keys::*;

mod keys;

/// Button layout similar to a Gameboy Advance.
#[derive(Debug, Clone, Copy, EnumCount, EnumIter)]
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
const KEY_STATE_BYTES: usize = 1024;

pub struct Input {
    devices: Vec<Device>,
    last_scanned: Instant,
    current_state: [bool; Button::COUNT],
    previous_state: [bool; Button::COUNT],
}

impl Default for Input {
    fn default() -> Self {
        let devices = Self::scan_devices();
        log::info!("found {} input devices", devices.len());
        Self {
            devices,
            last_scanned: Instant::now(),
            current_state: [false; Button::COUNT],
            previous_state: [false; Button::COUNT],
        }
    }
}

ioctl_read_buf!(key_state, b'E', 0x18, u8);
ioctl_read!(abs_x, b'E', 0x40 + ABS_X, InputAbsInfo);
ioctl_read!(abs_y, b'E', 0x40 + ABS_Y, InputAbsInfo);
ioctl_read!(abs_hat0x, b'E', 0x40 + ABS_HAT0X, InputAbsInfo);
ioctl_read!(abs_hat0y, b'E', 0x40 + ABS_HAT0Y, InputAbsInfo);

impl Input {
    fn scan_devices() -> Vec<Device> {
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

            let Some(kind) = DeviceKind::from_device(&file) else {
                continue;
            };

            devices.push(Device { file, kind });
        }
        devices
    }

    /// Updates the input state.
    /// Should be called once per game loop iteration, usually at the start.
    pub fn update(&mut self) {
        if self.last_scanned.elapsed() > SCAN_INTERVAL {
            self.devices = Self::scan_devices();
            self.last_scanned = Instant::now();
        }

        self.previous_state = self.current_state;
        self.current_state = [false; Button::COUNT];

        for device in &self.devices {
            device.poll(&mut self.current_state);
        }
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
struct Device {
    file: File,
    kind: DeviceKind,
}

impl Device {
    fn poll(&self, state: &mut [bool; Button::COUNT]) {
        if self.kind.has_keys() {
            self.poll_keys(state);
        }

        if self.kind.has_abs() {
            self.poll_abs(state);
        }
    }

    fn poll_keys(&self, state: &mut [bool; Button::COUNT]) {
        let mut bits = [0u8; KEY_STATE_BYTES];
        if unsafe { key_state(self.file.as_raw_fd(), &mut bits) }.is_err() {
            return;
        }

        if Self::has_bit(&bits, KEY_ESC) {
            std::process::exit(0);
        }

        state[Button::Up.index()] |=
            Self::has_bit(&bits, KEY_UP) || Self::has_bit(&bits, BTN_DPAD_UP);
        state[Button::Down.index()] |=
            Self::has_bit(&bits, KEY_DOWN) || Self::has_bit(&bits, BTN_DPAD_DOWN);
        state[Button::Left.index()] |=
            Self::has_bit(&bits, KEY_LEFT) || Self::has_bit(&bits, BTN_DPAD_LEFT);
        state[Button::Right.index()] |=
            Self::has_bit(&bits, KEY_RIGHT) || Self::has_bit(&bits, BTN_DPAD_RIGHT);

        state[Button::A.index()] |= Self::has_bit(&bits, KEY_A) || Self::has_bit(&bits, BTN_SOUTH);
        state[Button::B.index()] |= Self::has_bit(&bits, KEY_B) || Self::has_bit(&bits, BTN_EAST);
        state[Button::Start.index()] |=
            Self::has_bit(&bits, KEY_DOT) || Self::has_bit(&bits, BTN_START);
        state[Button::Select.index()] |=
            Self::has_bit(&bits, KEY_COMMA) || Self::has_bit(&bits, BTN_SELECT);
        state[Button::L.index()] |= Self::has_bit(&bits, KEY_L) || Self::has_bit(&bits, BTN_TL);
        state[Button::R.index()] |= Self::has_bit(&bits, KEY_R) || Self::has_bit(&bits, BTN_TR);
    }

    fn poll_abs(&self, state: &mut [bool; Button::COUNT]) {
        if let Some(value) = self.read_abs_x() {
            AxisValue {
                axis: Axis::X,
                value,
            }
            .apply(state);
        }

        if let Some(value) = self.read_abs_y() {
            AxisValue {
                axis: Axis::Y,
                value,
            }
            .apply(state);
        }

        if let Some(value) = self.read_abs_hat0x() {
            AxisValue {
                axis: Axis::X,
                value: value * THRESHOLD,
            }
            .apply(state);
        }

        if let Some(value) = self.read_abs_hat0y() {
            AxisValue {
                axis: Axis::Y,
                value: value * THRESHOLD,
            }
            .apply(state);
        }
    }

    fn read_abs_x(&self) -> Option<i32> {
        let mut info = InputAbsInfo::default();
        unsafe { abs_x(self.file.as_raw_fd(), &mut info) }
            .ok()
            .map(|_| info.value)
    }

    fn read_abs_y(&self) -> Option<i32> {
        let mut info = InputAbsInfo::default();
        unsafe { abs_y(self.file.as_raw_fd(), &mut info) }
            .ok()
            .map(|_| info.value)
    }

    fn read_abs_hat0x(&self) -> Option<i32> {
        let mut info = InputAbsInfo::default();
        unsafe { abs_hat0x(self.file.as_raw_fd(), &mut info) }
            .ok()
            .map(|_| info.value)
    }

    fn read_abs_hat0y(&self) -> Option<i32> {
        let mut info = InputAbsInfo::default();
        unsafe { abs_hat0y(self.file.as_raw_fd(), &mut info) }
            .ok()
            .map(|_| info.value)
    }

    fn has_bit(bits: &[u8], bit: u16) -> bool {
        let byte = bits[(bit / 8) as usize];
        let mask = 1 << (bit % 8);
        byte & mask != 0
    }
}

#[derive(Debug, Clone, Copy)]
enum DeviceKind {
    Keys,
    Abs,
    KeysAndAbs,
}

impl DeviceKind {
    fn from_device(file: &File) -> Option<Self> {
        let mut bits = [0u8; KEY_STATE_BYTES];
        let has_keys = unsafe { key_state(file.as_raw_fd(), &mut bits) }.is_ok();

        let mut info = InputAbsInfo::default();
        let has_abs = unsafe { abs_x(file.as_raw_fd(), &mut info) }.is_ok()
            || unsafe { abs_y(file.as_raw_fd(), &mut info) }.is_ok()
            || unsafe { abs_hat0x(file.as_raw_fd(), &mut info) }.is_ok()
            || unsafe { abs_hat0y(file.as_raw_fd(), &mut info) }.is_ok();

        match (has_keys, has_abs) {
            (true, true) => Some(Self::KeysAndAbs),
            (true, false) => Some(Self::Keys),
            (false, true) => Some(Self::Abs),
            (false, false) => None,
        }
    }

    fn has_keys(self) -> bool {
        matches!(self, Self::Keys | Self::KeysAndAbs)
    }

    fn has_abs(self) -> bool {
        matches!(self, Self::Abs | Self::KeysAndAbs)
    }
}

#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
struct InputAbsInfo {
    value: i32,
    minimum: i32,
    maximum: i32,
    fuzz: i32,
    flat: i32,
    resolution: i32,
}

#[derive(Debug)]
struct AxisValue {
    axis: Axis,
    value: i32,
}

impl AxisValue {
    fn apply(&self, button_state: &mut [bool; Button::COUNT]) {
        let (negative, positive) = self.axis.buttons();
        button_state[negative.index()] |= self.value <= -DEADZONE;
        button_state[positive.index()] |= self.value >= DEADZONE;
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
