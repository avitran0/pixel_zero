use thiserror::Error;

#[derive(Debug, Error)]
pub enum FontError {}

pub struct Font {}

impl Font {
    pub fn load() -> Result<Self, FontError> {
        Ok(Self {})
    }
}
