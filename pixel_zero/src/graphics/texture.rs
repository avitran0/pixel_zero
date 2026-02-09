use std::{io::Cursor, path::Path};

use glam::{UVec2, uvec2};
use image::{ImageReader, RgbaImage};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum TextureError {
    #[error("I/O Error: {0}")]
    IO(#[from] std::io::Error),
    #[error("Image Decoding: {0}")]
    Image(#[from] image::ImageError),
}

pub struct Texture {
    texture: u32,
    size: UVec2,
}

impl Texture {
    pub fn load(path: impl AsRef<Path>) -> Result<Self, TextureError> {
        let image = ImageReader::open(path)?.decode()?;
        let rgba_image = image.to_rgba8();
        let size = uvec2(image.width(), image.height());

        let texture = Self::crate_texture(size, Some(rgba_image));

        Ok(Self { texture, size })
    }

    pub fn load_binary(bytes: &[u8]) -> Result<Self, TextureError> {
        let cursor = Cursor::new(bytes);
        let image = ImageReader::new(cursor).with_guessed_format()?.decode()?;
        let rgba_image = image.to_rgba8();
        let size = uvec2(image.width(), image.height());

        let texture = Self::crate_texture(size, Some(rgba_image));

        Ok(Self { texture, size })
    }

    pub fn empty(size: UVec2) -> Self {
        let texture = Self::crate_texture(size, None);
        Self { texture, size }
    }

    fn crate_texture(size: UVec2, data: Option<RgbaImage>) -> u32 {
        let mut texture = 0;
        unsafe {
            gl::GenTextures(1, &raw mut texture);
            gl::BindTexture(gl::TEXTURE_2D, texture);

            gl::TexParameteri(
                gl::TEXTURE_2D,
                gl::TEXTURE_MIN_FILTER,
                gl::NEAREST.cast_signed(),
            );
            gl::TexParameteri(
                gl::TEXTURE_2D,
                gl::TEXTURE_MAG_FILTER,
                gl::NEAREST.cast_signed(),
            );
            gl::TexParameteri(
                gl::TEXTURE_2D,
                gl::TEXTURE_WRAP_S,
                gl::CLAMP_TO_EDGE.cast_signed(),
            );
            gl::TexParameteri(
                gl::TEXTURE_2D,
                gl::TEXTURE_WRAP_T,
                gl::CLAMP_TO_EDGE.cast_signed(),
            );

            let pixels = if let Some(data) = data {
                data.as_raw().as_ptr().cast()
            } else {
                std::ptr::null()
            };
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGBA.cast_signed(),
                size.x.cast_signed(),
                size.y.cast_signed(),
                0,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                pixels,
            );

            gl::BindTexture(gl::TEXTURE_2D, 0);
        }

        texture
    }

    pub fn bind(&self) {
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, self.texture);
        }
    }

    pub fn unbind() {
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, 0);
        }
    }

    pub const fn size(&self) -> UVec2 {
        self.size
    }

    pub(crate) const fn handle(&self) -> u32 {
        self.texture
    }
}
