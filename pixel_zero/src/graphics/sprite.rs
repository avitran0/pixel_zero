use std::{path::Path, sync::Arc};

use glam::{UVec2, Vec2, Vec4, vec2, vec4};

use crate::graphics::texture::{Texture, TextureError};

#[derive(Debug, Clone)]
pub struct Sprite(Arc<SpriteInner>);

impl Sprite {
    pub(crate) fn load(gl: &glow::Context, path: impl AsRef<Path>) -> Result<Self, TextureError> {
        let inner = SpriteInner::load(gl, path)?;
        Ok(Self(Arc::new(inner)))
    }

    pub(crate) fn load_binary_png(gl: &glow::Context, bytes: &[u8]) -> Result<Self, TextureError> {
        let inner = SpriteInner::load_binary_png(gl, bytes)?;
        Ok(Self(Arc::new(inner)))
    }

    pub(crate) fn texture(&self) -> &Texture {
        &self.0.texture
    }

    pub(crate) fn region(&self) -> &TextureRegion {
        &self.0.region
    }
}

#[derive(Debug, Clone)]
struct SpriteInner {
    texture: Arc<Texture>,
    region: TextureRegion,
}

impl SpriteInner {
    fn load(gl: &glow::Context, path: impl AsRef<Path>) -> Result<Self, TextureError> {
        let texture = Arc::new(Texture::load(gl, path)?);
        let region = TextureRegion::full();
        Ok(Self { texture, region })
    }

    fn load_binary_png(gl: &glow::Context, bytes: &[u8]) -> Result<Self, TextureError> {
        let texture = Arc::new(Texture::load_binary_png(gl, bytes)?);
        let region = TextureRegion::full();
        Ok(Self { texture, region })
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct TextureRegion {
    min: Vec2,
    max: Vec2,
}

impl TextureRegion {
    pub(crate) fn from_pixels(position: UVec2, size: UVec2, texture_size: UVec2) -> Self {
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

    #[allow(dead_code)]
    pub(crate) fn min(&self) -> Vec2 {
        self.min
    }

    #[allow(dead_code)]
    pub(crate) fn max(&self) -> Vec2 {
        self.max
    }

    pub(crate) fn vec4(&self) -> Vec4 {
        vec4(self.min.x, self.min.y, self.max.x, self.max.y)
    }
}
