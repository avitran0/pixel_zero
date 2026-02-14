use std::{io::Cursor, path::Path, sync::Arc};

use glam::{UVec2, uvec2};
use glow::{HasContext, NativeTexture};
use image::ImageReader;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum TextureError {
    #[error("OpenGL error: {0}")]
    OpenGL(String),
    #[error("I/O error: {0}")]
    IO(#[from] std::io::Error),
    #[error("Image decoding: {0}")]
    Image(#[from] image::ImageError),
}

#[derive(Debug)]
pub struct Texture(Arc<TextureInner>);

impl Texture {
    pub(crate) fn load(gl: &glow::Context, path: impl AsRef<Path>) -> Result<Self, TextureError> {
        let inner = TextureInner::load(gl, path)?;
        Ok(Self(Arc::new(inner)))
    }

    pub(crate) fn load_binary_png(gl: &glow::Context, data: &[u8]) -> Result<Self, TextureError> {
        let inner = TextureInner::load_binary_png(gl, data)?;
        Ok(Self(Arc::new(inner)))
    }

    pub(crate) fn load_rgba(
        gl: &glow::Context,
        data: &[u8],
        size: UVec2,
    ) -> Result<Self, TextureError> {
        let inner = TextureInner::load_rgba(gl, data, size)?;
        Ok(Self(Arc::new(inner)))
    }

    pub(crate) fn load_empty(gl: &glow::Context, size: UVec2) -> Result<Self, TextureError> {
        let inner = TextureInner::load_empty(gl, size)?;
        Ok(Self(Arc::new(inner)))
    }

    pub(crate) fn bind(&self, gl: &glow::Context) {
        unsafe {
            gl.bind_texture(glow::TEXTURE_2D, Some(self.0.texture));
        }
    }

    pub(crate) fn unbind(gl: &glow::Context) {
        unsafe {
            gl.bind_texture(glow::TEXTURE_2D, None);
        }
    }

    #[must_use]
    pub fn size(&self) -> UVec2 {
        self.0.size
    }

    pub(crate) fn handle(&self) -> NativeTexture {
        self.0.texture
    }
}

#[derive(Debug)]
struct TextureInner {
    texture: NativeTexture,
    size: UVec2,
}

impl TextureInner {
    fn load(gl: &glow::Context, path: impl AsRef<Path>) -> Result<Self, TextureError> {
        let image = ImageReader::open(path)?.decode()?;
        let rgba_image = image.to_rgba8();
        let size = uvec2(image.width(), image.height());

        let texture = Self::create_texture(gl, size, Some(rgba_image.as_raw()))?;

        Ok(Self { texture, size })
    }

    fn load_binary_png(gl: &glow::Context, data: &[u8]) -> Result<Self, TextureError> {
        let cursor = Cursor::new(data);
        let image = ImageReader::new(cursor).with_guessed_format()?.decode()?;
        let rgba_image = image.to_rgba8();
        let size = uvec2(image.width(), image.height());

        let texture = Self::create_texture(gl, size, Some(rgba_image.as_raw()))?;

        Ok(Self { texture, size })
    }

    fn load_rgba(gl: &glow::Context, data: &[u8], size: UVec2) -> Result<Self, TextureError> {
        let texture = Self::create_texture(gl, size, Some(data))?;
        Ok(Self { texture, size })
    }

    fn load_empty(gl: &glow::Context, size: UVec2) -> Result<Self, TextureError> {
        let texture = Self::create_texture(gl, size, None)?;
        Ok(Self { texture, size })
    }

    fn create_texture(
        gl: &glow::Context,
        size: UVec2,
        data: Option<&[u8]>,
    ) -> Result<NativeTexture, TextureError> {
        let texture = unsafe { gl.create_texture().map_err(TextureError::OpenGL)? };

        unsafe {
            gl.bind_texture(glow::TEXTURE_2D, Some(texture));
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_MIN_FILTER,
                glow::NEAREST.cast_signed(),
            );
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_MAG_FILTER,
                glow::NEAREST.cast_signed(),
            );
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_WRAP_S,
                glow::CLAMP_TO_EDGE.cast_signed(),
            );
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_WRAP_T,
                glow::CLAMP_TO_EDGE.cast_signed(),
            );

            gl.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                glow::RGBA8.cast_signed(),
                size.x.cast_signed(),
                size.y.cast_signed(),
                0,
                glow::RGBA,
                glow::UNSIGNED_BYTE,
                glow::PixelUnpackData::Slice(data),
            );
            gl.bind_texture(glow::TEXTURE_2D, None);
        }

        Ok(texture)
    }
}
