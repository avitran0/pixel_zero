use std::{
    collections::HashMap,
    fs::File,
    io::{BufReader, Read},
    path::Path,
};

use bytemuck::{AnyBitPattern, NoUninit};
use glam::{UVec2, uvec2};
use thiserror::Error;

use crate::{
    graphics::{sprite::TextureRegion, texture::Texture},
    io::ReadBytes,
};

#[derive(Debug, Error)]
pub enum FontError {
    #[error("I/O Error: {0}")]
    IO(#[from] std::io::Error),
    #[error("File magic is 0x{expected:X}, but is 0x{actual:X}")]
    InvalidMagic { expected: u32, actual: u32 },
    #[error("File version should be 0, but is {0}")]
    InvalidVersion(u32),
    #[error("Invalid unicode codepoint")]
    InvalidUnicode,
}

pub struct Font {
    texture: Texture,
    glyph_size: UVec2,
    glyphs: Vec<Glyph>,
    char_map: Option<HashMap<char, usize>>,
}

impl Font {
    pub fn load(path: impl AsRef<Path>) -> Result<Self, FontError> {
        let file = File::open(path)?;
        let mut reader = BufReader::new(file);

        let header: Header = reader.read_value()?;
        if header.magic != Header::MAGIC {
            return Err(FontError::InvalidMagic {
                expected: Header::MAGIC,
                actual: header.magic,
            });
        }

        if header.version != 0 {
            return Err(FontError::InvalidVersion(header.version));
        }

        let has_unicode = header.flags & 0x01 != 0;
        let glyph_bytes = header.bytes_per_glyph * header.num_glyphs;
        let glyph_data = reader.read_bytes(glyph_bytes as usize)?;

        let (atlas_size, glyphs_per_row) = Self::calculate_atlas_dimensions(&header);

        let mut atlas_data = vec![0xFF; atlas_size.x as usize * atlas_size.y as usize * 4];
        let mut glyphs = Vec::with_capacity(header.num_glyphs as usize);

        for glyph in 0..header.num_glyphs {
            let x = (glyph % glyphs_per_row) * header.width;
            let y = (glyph / glyphs_per_row) * header.height;

            let bpg = header.bytes_per_glyph as usize;
            let glyph_offset = glyph as usize * bpg;
            let glyph_slice = &glyph_data[glyph_offset..glyph_offset + bpg];

            let width = Self::blit_glyph_to_atlas(
                glyph_slice,
                &mut atlas_data,
                uvec2(header.width, header.height),
                uvec2(x, y),
                atlas_size.x,
            );

            let region = TextureRegion::from_pixels(
                uvec2(x, y),
                uvec2(header.width, header.height),
                atlas_size,
            );

            glyphs.push(Glyph {
                region,
                advance: width + 1,
            });
        }

        let char_map = if has_unicode {
            Some(Self::parse_unicode_table(
                &mut reader,
                header.num_glyphs as usize,
            )?)
        } else {
            None
        };

        let space_index = if let Some(char_map) = &char_map {
            char_map.get(&' ').cloned().unwrap_or(' ' as usize)
        } else {
            ' ' as usize
        };

        if let Some(space) = glyphs.get_mut(space_index) {
            space.advance = header.width / 2;
        }

        let texture = Texture::from_rgba(&atlas_data, atlas_size);

        log::info!("loaded font with {} glyphs", glyphs.len());

        Ok(Self {
            texture,
            glyph_size: uvec2(header.width, header.height),
            glyphs,
            char_map,
        })
    }

    fn calculate_atlas_dimensions(header: &Header) -> (UVec2, u32) {
        let total_area = header.num_glyphs * header.width * header.height;
        let side_size = (total_area as f32).sqrt().ceil() as u32;

        let glyphs_per_row = (side_size / header.width).max(1);
        let rows = (header.num_glyphs as f32 / glyphs_per_row as f32).ceil() as u32;

        let atlas_size = uvec2(glyphs_per_row * header.width, rows * header.height);
        let atlas_size = uvec2(
            atlas_size.x.next_power_of_two(),
            atlas_size.y.next_power_of_two(),
        );

        (atlas_size, glyphs_per_row)
    }

    fn blit_glyph_to_atlas(
        glyph_data: &[u8],
        atlas_data: &mut [u8],
        size: UVec2,
        position: UVec2,
        atlas_width: u32,
    ) -> u32 {
        let bytes_per_row = size.x.div_ceil(8);
        let mut max_width = 0;

        for y in 0..size.y {
            // calculate maximum line

            for x in 0..size.x {
                let byte_index = (y * bytes_per_row + x / 8) as usize;
                let bit_index = 7 - (x % 8);
                let bit = ((glyph_data[byte_index] >> bit_index) & 1) != 0;
                if bit && x > max_width {
                    max_width = x;
                }

                let atlas_index = ((position.y + y) * atlas_width + (position.x + x)) * 4;
                let atlas_index = atlas_index as usize;

                // only set alpha, rest is 0xFF already
                atlas_data[atlas_index + 3] = if bit { 0xFF } else { 0x00 };
            }
        }

        max_width
    }

    fn parse_unicode_table(
        reader: &mut impl Read,
        num_glyphs: usize,
    ) -> Result<HashMap<char, usize>, FontError> {
        // read whole data in one go
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf)?;

        let lines = buf.split(|&b| b == 0xFF);

        let mut unicode_map = HashMap::new();

        for (glyph_index, line) in lines.enumerate() {
            if glyph_index >= num_glyphs {
                break;
            }

            if let Ok(s) = str::from_utf8(line)
                && let Some(c) = s.chars().next()
            {
                unicode_map.insert(c, glyph_index);
            }
        }

        Ok(unicode_map)
    }

    pub(crate) fn texture(&self) -> &Texture {
        &self.texture
    }

    pub(crate) fn glyph_size(&self) -> UVec2 {
        self.glyph_size
    }

    pub(crate) fn glyph(&self, c: char) -> Option<&Glyph> {
        let index = if let Some(char_map) = &self.char_map {
            let index = char_map.get(&c)?;
            *index
        } else {
            c as usize
        };

        self.glyphs.get(index)
    }

    pub(crate) fn default_glyph(&self) -> &Glyph {
        &self.glyphs[0]
    }
}

pub(crate) struct Glyph {
    region: TextureRegion,
    advance: u32,
}

impl Glyph {
    pub(crate) fn region(&self) -> &TextureRegion {
        &self.region
    }

    pub(crate) fn advance(&self) -> u32 {
        self.advance
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
