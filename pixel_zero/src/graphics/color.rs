use glam::{Vec3, Vec4, vec3, vec4};

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
    pub fn vec3(&self) -> Vec3 {
        self.f32().vec3()
    }

    #[must_use]
    pub fn vec4(&self) -> Vec4 {
        self.f32().vec4()
    }

    pub(crate) fn f32(self) -> ColorF32 {
        ColorF32 {
            r: f32::from(self.r) / 255.0,
            g: f32::from(self.g) / 255.0,
            b: f32::from(self.b) / 255.0,
            a: f32::from(self.a) / 255.0,
        }
    }
}

pub(crate) struct ColorF32 {
    r: f32,
    g: f32,
    b: f32,
    a: f32,
}

impl ColorF32 {
    pub(crate) fn r(&self) -> f32 {
        self.r
    }

    pub(crate) fn g(&self) -> f32 {
        self.g
    }

    pub(crate) fn b(&self) -> f32 {
        self.b
    }

    pub(crate) fn a(&self) -> f32 {
        self.a
    }

    pub(crate) fn vec3(&self) -> Vec3 {
        vec3(self.r, self.g, self.b)
    }

    pub(crate) fn vec4(&self) -> Vec4 {
        vec4(self.r, self.g, self.b, self.a)
    }
}
