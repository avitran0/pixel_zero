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

    #[must_use]
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    #[must_use]
    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    #[must_use]
    pub fn r(&self) -> u8 {
        self.r
    }

    #[must_use]
    pub fn g(&self) -> u8 {
        self.g
    }

    #[must_use]
    pub fn b(&self) -> u8 {
        self.b
    }

    #[must_use]
    pub fn a(&self) -> u8 {
        self.a
    }

    #[must_use]
    pub fn colorf32(&self) -> ColorF32 {
        ColorF32::from(self)
    }
}

impl From<ColorF32> for Color {
    fn from(value: ColorF32) -> Self {
        Self {
            r: (value.r * 255.0) as u8,
            g: (value.g * 255.0) as u8,
            b: (value.b * 255.0) as u8,
            a: (value.a * 255.0) as u8,
        }
    }
}

impl From<&ColorF32> for Color {
    fn from(value: &ColorF32) -> Self {
        Self {
            r: (value.r * 255.0) as u8,
            g: (value.g * 255.0) as u8,
            b: (value.b * 255.0) as u8,
            a: (value.a * 255.0) as u8,
        }
    }
}

pub struct ColorF32 {
    r: f32,
    g: f32,
    b: f32,
    a: f32,
}

impl ColorF32 {
    pub const BLACK: Self = Self::rgb(0.0, 0.0, 0.0);
    pub const WHITE: Self = Self::rgb(1.0, 1.0, 1.0);

    #[must_use]
    pub const fn rgb(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b, a: 1.0 }
    }

    #[must_use]
    pub const fn rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    #[must_use]
    pub fn r(&self) -> f32 {
        self.r
    }

    #[must_use]
    pub fn g(&self) -> f32 {
        self.g
    }

    #[must_use]
    pub fn b(&self) -> f32 {
        self.b
    }

    #[must_use]
    pub fn a(&self) -> f32 {
        self.a
    }

    #[must_use]
    pub fn color(&self) -> Color {
        Color::from(self)
    }
}

impl From<Color> for ColorF32 {
    fn from(value: Color) -> Self {
        Self {
            r: f32::from(value.r) / 255.0,
            g: f32::from(value.g) / 255.0,
            b: f32::from(value.b) / 255.0,
            a: f32::from(value.a) / 255.0,
        }
    }
}

impl From<&Color> for ColorF32 {
    fn from(value: &Color) -> Self {
        Self {
            r: f32::from(value.r) / 255.0,
            g: f32::from(value.g) / 255.0,
            b: f32::from(value.b) / 255.0,
            a: f32::from(value.a) / 255.0,
        }
    }
}
