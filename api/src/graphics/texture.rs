use std::path::Path;

use image::ImageReader;

pub struct Texture {
    texture: u32,
}

impl Texture {
    pub fn load(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let image = ImageReader::open(path)?.decode()?;

        let mut texture = 0;
        unsafe {
            gl::GenTextures(1, &mut texture);
            gl::BindTexture(gl::TEXTURE0, texture);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
        }

        Ok(Self { texture })
    }
}
