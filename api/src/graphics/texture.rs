use std::path::Path;

use glam::{UVec2, uvec2};
use image::ImageReader;

pub struct Texture {
    texture: u32,
    size: UVec2,
}

impl Texture {
    pub fn load(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let image = ImageReader::open(path)?.decode()?;
        let rgba_image = image.to_rgba8();
        let size = uvec2(image.width(), image.height());
        if !size.x.is_power_of_two() || !size.y.is_power_of_two() {
            return Err(anyhow::anyhow!(
                "texture size is not power of two: {}x{}",
                size.x,
                size.y
            ));
        }

        let mut texture = 0;
        unsafe {
            gl::GenTextures(1, &mut texture);
            gl::BindTexture(gl::TEXTURE0, texture);

            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);

            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGBA as i32,
                size.x as i32,
                size.y as i32,
                0,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                rgba_image.as_raw().as_ptr() as *const _,
            );

            gl::BindTexture(gl::TEXTURE_2D, 0);
        }

        Ok(Self { texture, size })
    }

    pub fn empty(size: UVec2) -> anyhow::Result<Self> {
        if !size.x.is_power_of_two() || !size.y.is_power_of_two() {
            return Err(anyhow::anyhow!(
                "texture size is not power of two: {}x{}",
                size.x,
                size.y
            ));
        }

        let mut texture = 0;
        unsafe {
            gl::GenTextures(1, &mut texture);
            gl::BindTexture(gl::TEXTURE0, texture);

            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);

            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGBA as i32,
                size.x as i32,
                size.y as i32,
                0,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                std::ptr::null(),
            );

            gl::BindTexture(gl::TEXTURE_2D, 0);
        }

        Ok(Self { texture, size })
    }

    pub fn bind(&self) {
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, self.texture);
        }
    }

    pub fn unbind(&self) {
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, 0);
        }
    }

    pub fn size(&self) -> UVec2 {
        self.size
    }

    pub(crate) fn handle(&self) -> u32 {
        self.texture
    }
}
