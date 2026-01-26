use std::path::Path;

use glam::Vec2;

use crate::graphics::texture::Texture;

pub struct Sprite {
    texture: Texture,
}

impl Sprite {
    pub fn load(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let texture = Texture::load(path)?;
        Ok(Self { texture })
    }

    pub fn draw(position: Vec2) {}
}
