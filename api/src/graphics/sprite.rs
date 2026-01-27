use std::path::Path;

use crate::graphics::texture::Texture;

pub struct Sprite {
    texture: Texture,
}

impl Sprite {
    pub fn load(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let texture = Texture::load(path)?;
        Ok(Self { texture })
    }
}
