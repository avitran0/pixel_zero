use std::io::Read;

use bytemuck::{AnyBitPattern, NoUninit};
use thiserror::Error;

use crate::io::ReadBytes;

#[derive(Debug, Error)]
pub enum FontError {
    #[error("I/O Error: {0}")]
    IO(#[from] std::io::Error),
    #[error("File magic is 0x864ab572, but is 0x{0:X}")]
    InvalidMagic(u32),
}

pub struct Font {}

impl Font {
    pub fn load(data: &mut impl Read) -> Result<Self, FontError> {
        let header: Header = data.read_value()?;
        if header.magic != Header::MAGIC {
            return Err(FontError::InvalidMagic(header.magic));
        }

        let has_unicode = header.flags & 0x01 != 0;
        Ok(Self {})
    }
}

#[repr(C)]
#[derive(Default, Clone, Copy, AnyBitPattern, NoUninit)]
struct Header {
    magic: u32,
    /// should be zero
    version: u32,
    /// should be 32
    header_size: u32,
    /// 0 if no unicode, 1 if unicode
    flags: u32,
    num_glyphs: u32,
    bytes_per_glyph: u32,
    height: u32,
    width: u32,
}

impl Header {
    const MAGIC: u32 = 0x864ab572;
}
