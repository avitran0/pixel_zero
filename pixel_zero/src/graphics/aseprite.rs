use std::{
    fs::File,
    io::{BufReader, Seek as _, SeekFrom},
    path::Path,
};

use thiserror::Error;

use crate::io::ReadBytes as _;

#[derive(Debug, Error)]
pub enum AsepriteError {
    #[error("I/O Error: {0}")]
    IO(#[from] std::io::Error),
    #[error("Invalid Magic (is 0x{0:X}, should be 0xA5E0)")]
    InvalidMagic(u16),
    #[error("Invalid Color Depth")]
    InvalidColorDepth(u16),
    #[error("Invalid Frame Magic (is 0x{0:X}, should be 0xF1FA)")]
    InvalidFrameMagic(u16),
}

pub struct AsepriteImage {}

impl AsepriteImage {
    pub fn load(path: impl AsRef<Path>) -> Result<Self, AsepriteError> {
        let file = File::open(path)?;
        let mut reader = BufReader::new(file);

        let file_size = reader.read_u32()?;
        let magic = reader.read_u16()?;
        if magic != 0xA5E0 {
            return Err(AsepriteError::InvalidMagic(magic));
        }

        let frame_count = reader.read_u16()?;
        let width = reader.read_u16()?;
        let height = reader.read_u16()?;

        let color_depth = reader.read_u16()?;
        let color_depth = match color_depth {
            32 => ColorDepth::RGBA,
            16 => ColorDepth::Grayscale,
            8 => ColorDepth::Indexed,
            _ => return Err(AsepriteError::InvalidColorDepth(color_depth)),
        };

        reader.seek(SeekFrom::Start(128))?;

        let mut frames: Vec<Frame> = Vec::with_capacity(frame_count as usize);
        for frame in 0..frame_count {
            let frame_size = reader.read_u32()?;
            let frame_magic = reader.read_u16()?;
            if frame_magic != 0xF1FA {
                return Err(AsepriteError::InvalidFrameMagic(frame_magic));
            }

            let old_chunk_count = reader.read_u16()?;
            reader.seek(SeekFrom::Current(4))?;
            let new_chunk_count = reader.read_u32()?;
            let chunk_count = if new_chunk_count == 0 {
                old_chunk_count as u32
            } else {
                new_chunk_count
            };
        }

        Err(AsepriteError::InvalidFrameMagic(0))
    }
}

enum ColorDepth {
    RGBA,
    Grayscale,
    Indexed,
}

enum ChunkType {
    Palette,
}

struct Frame {}
