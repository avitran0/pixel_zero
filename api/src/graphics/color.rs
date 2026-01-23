#[derive(Debug, Clone, Copy)]
pub struct Color {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

impl Color {
    pub const BLACK: Self = Self::rgb(0, 0, 0);
    pub const WHITE: Self = Self::rgb(255, 255, 255);
    pub const RED: Self = Self::rgb(255, 0, 0);
    pub const GREEN: Self = Self::rgb(0, 255, 0);
    pub const BLUE: Self = Self::rgb(0, 0, 255);

    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    /// Convert color to RGBA array (0.0-1.0 range for OpenGL)
    pub fn as_f32_array(&self) -> [f32; 4] {
        [
            self.r as f32 / 255.0,
            self.g as f32 / 255.0,
            self.b as f32 / 255.0,
            self.a as f32 / 255.0,
        ]
    }

    /// Convert color to RGBA u8 array
    pub fn as_u8_array(&self) -> [u8; 4] {
        [self.r, self.g, self.b, self.a]
    }

    /// Get the red component
    pub fn r(&self) -> u8 {
        self.r
    }

    /// Get the green component
    pub fn g(&self) -> u8 {
        self.g
    }

    /// Get the blue component
    pub fn b(&self) -> u8 {
        self.b
    }

    /// Get the alpha component
    pub fn a(&self) -> u8 {
        self.a
    }
}
