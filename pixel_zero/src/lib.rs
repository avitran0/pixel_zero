mod ffi;
pub mod graphics;
pub mod input;
pub mod io;
pub mod log;
pub mod meta;
pub mod terminal;

pub use glam::{IVec2, ivec2};
pub use graphics::{Graphics, font::Font, sprite::Sprite};

pub const WIDTH: u32 = 320;
pub const HEIGHT: u32 = 240;
