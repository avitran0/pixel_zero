//! # Pixel Zero
//!
//! This is a game library for embedded linux systems.

mod ffi;
pub mod graphics;
pub mod input;
pub mod io;
pub mod log;
pub mod meta;
mod terminal;
pub mod ui;

pub use glam;

/// Framebuffer width
pub const WIDTH: u32 = 320;
/// Framebuffer  height
pub const HEIGHT: u32 = 180;
