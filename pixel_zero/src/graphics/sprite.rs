use std::path::Path;

use crate::graphics::texture::{Texture, TextureError};

pub struct Sprite {
    pub(crate) texture: Texture,
}

impl Sprite {
    pub fn load(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let texture = Texture::load(path)?;
        Ok(Self { texture })
    }

    pub fn load_binary(bytes: &[u8]) -> Result<Self, TextureError> {
        let texture = Texture::load_binary(bytes)?;
        Ok(Self { texture })
    }
}
