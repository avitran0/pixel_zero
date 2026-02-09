use std::path::Path;

use glam::{UVec2, Vec2, vec2};

use crate::graphics::texture::{Texture, TextureError};

pub struct Sprite {
    texture: Texture,
    region: TextureRegion,
}

impl Sprite {
    pub fn load(path: impl AsRef<Path>) -> Result<Self, TextureError> {
        let texture = Texture::load(path)?;
        let region = TextureRegion::full();
        Ok(Self { texture, region })
    }

    pub fn load_binary(bytes: &[u8]) -> Result<Self, TextureError> {
        let texture = Texture::load_binary(bytes)?;
        let region = TextureRegion::full();
        Ok(Self { texture, region })
    }

    pub(crate) fn texture(&self) -> &Texture {
        &self.texture
    }

    pub(crate) fn region(&self) -> &TextureRegion {
        &self.region
    }
}

pub(crate) struct TextureRegion {
    min: Vec2,
    max: Vec2,
}

impl TextureRegion {
    fn from_pixels(position: UVec2, size: UVec2, texture_size: UVec2) -> Self {
        let position = position.as_vec2();
        let size = size.as_vec2();
        let texture_size = texture_size.as_vec2();

        let min = vec2(position.x / texture_size.x, position.y / texture_size.y);
        let max = vec2(
            (position.x + size.x) / texture_size.x,
            (position.y + size.y) / texture_size.y,
        );

        Self { min, max }
    }

    fn full() -> Self {
        Self {
            min: Vec2::ZERO,
            max: Vec2::ONE,
        }
    }

    pub(crate) fn min(&self) -> Vec2 {
        self.min
    }

    pub(crate) fn max(&self) -> Vec2 {
        self.max
    }
}
