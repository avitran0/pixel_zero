use crate::graphics::color::Color;

/// A 2D sprite with pixel data
#[derive(Debug, Clone)]
pub struct Sprite {
    width: u32,
    height: u32,
    pixels: Vec<Color>,
}

impl Sprite {
    /// Create a new sprite with the given dimensions
    pub fn new(width: u32, height: u32) -> Self {
        let pixels = vec![Color::rgba(0, 0, 0, 0); (width * height) as usize];
        Self {
            width,
            height,
            pixels,
        }
    }

    /// Create a sprite from raw pixel data
    pub fn from_pixels(width: u32, height: u32, pixels: Vec<Color>) -> Option<Self> {
        if pixels.len() != (width * height) as usize {
            return None;
        }
        Some(Self {
            width,
            height,
            pixels,
        })
    }

    /// Get the width of the sprite
    pub fn width(&self) -> u32 {
        self.width
    }

    /// Get the height of the sprite
    pub fn height(&self) -> u32 {
        self.height
    }

    /// Get pixel at position (x, y)
    pub fn get_pixel(&self, x: u32, y: u32) -> Option<Color> {
        if x >= self.width || y >= self.height {
            return None;
        }
        let index = (y * self.width + x) as usize;
        self.pixels.get(index).copied()
    }

    /// Set pixel at position (x, y)
    pub fn set_pixel(&mut self, x: u32, y: u32, color: Color) {
        if x >= self.width || y >= self.height {
            return;
        }
        let index = (y * self.width + x) as usize;
        if let Some(pixel) = self.pixels.get_mut(index) {
            *pixel = color;
        }
    }

    /// Get reference to the pixel data
    pub fn pixels(&self) -> &[Color] {
        &self.pixels
    }
}
