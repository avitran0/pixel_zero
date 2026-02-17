use std::{
    io::Read,
    path::Path,
    sync::atomic::{AtomicBool, Ordering},
    time::{Duration, Instant},
};

use ::drm::control::{Device as _, PageFlipFlags, framebuffer as drmfb};
use ::gbm::{BufferObject, FrontBufferError};
use thiserror::Error;

use crate::{
    graphics::{
        drm::{Drm, DrmError},
        egl::Egl,
        font::FontError,
        framebuffer::{Framebuffer, FramebufferError},
        gbm::Gbm,
        shader::ShaderError,
        texture::TextureError,
    },
    terminal::TerminalGuard,
};

pub use crate::graphics::{
    color::Color, font::Font, frame::Frame, sprite::Sprite, texture::Texture,
};

pub mod color;
mod drm;
mod egl;
pub mod font;
pub mod frame;
mod framebuffer;
mod gbm;
pub mod line;
mod quad;
mod shader;
pub mod sprite;
mod texture;

#[derive(Debug, Error)]
pub enum GraphicsError {
    #[error("I/O Error: {0}")]
    IO(#[from] std::io::Error),
    #[error("DRM Error: {0}")]
    Drm(#[from] DrmError),
    #[error("EGL Error: {0}")]
    Egl(#[from] khronos_egl::Error),
    #[error("{0}")]
    Shader(#[from] ShaderError),
    #[error("{0}")]
    Framebuffer(#[from] FramebufferError),
    #[error("Front Buffer Error: {0}")]
    FrontBuffer(#[from] FrontBufferError),
    #[error("Graphics is already loaded")]
    AlreadyLoaded,
}

pub struct Graphics {
    // this needs to be first to be dropped first
    framebuffer: Framebuffer,
    frame_start: Instant,
    fps_timer: Instant,
    fps_frames: u32,
    fps: u32,

    drm_fb: drmfb::Handle,
    buffer_object: BufferObject<()>,

    egl: Egl,
    gbm: Gbm,
    drm: Drm,

    _terminal_guard: TerminalGuard,
}

pub(crate) static GRAPHICS_LOADED: AtomicBool = AtomicBool::new(false);
impl Graphics {
    pub fn load() -> Result<Self, GraphicsError> {
        if GRAPHICS_LOADED.swap(true, Ordering::Relaxed) {
            return Err(GraphicsError::AlreadyLoaded);
        }

        let terminal_guard = TerminalGuard::new().map_err(std::io::Error::from)?;

        let drm = Drm::load()?;
        let mut gbm = Gbm::load(&drm)?;
        let egl = Egl::load(&mut gbm)?;

        let buffer_object = unsafe { gbm.surface().lock_front_buffer() }?;
        let bpp = buffer_object.bpp();
        let drm_fb = drm.gpu().add_framebuffer(&buffer_object, bpp, bpp)?;
        drm.gpu().set_crtc(
            drm.crtc().handle(),
            Some(drm_fb),
            (0, 0),
            &[drm.connector().handle()],
            Some(*drm.mode()),
        )?;

        let framebuffer = Framebuffer::load(egl.gl(), drm.size())?;
        let frame_start = Instant::now();
        let fps_timer = frame_start;

        Ok(Self {
            framebuffer,
            frame_start,
            fps_timer,
            fps_frames: 0,
            fps: 0,
            drm_fb,
            buffer_object,
            drm,
            gbm,
            egl,
            _terminal_guard: terminal_guard,
        })
    }

    pub fn load_sprite(&self, path: impl AsRef<Path>) -> Result<Sprite, TextureError> {
        Sprite::load(self.egl.gl(), path)
    }

    pub fn load_sprite_binary_png(&self, data: &[u8]) -> Result<Sprite, TextureError> {
        Sprite::load_binary_png(self.egl.gl(), data)
    }

    pub fn load_font(&self, path: impl AsRef<Path>) -> Result<Font, FontError> {
        Font::load(self.egl.gl(), path)
    }

    pub fn load_font_binary(&self, data: &[u8]) -> Result<Font, FontError> {
        Font::load_binary(self.egl.gl(), data)
    }

    pub fn load_font_read(&self, reader: &mut impl Read) -> Result<Font, FontError> {
        Font::load_read(self.egl.gl(), reader)
    }

    const FRAME_DURATION: Duration = Duration::from_micros(16667);
    pub fn present_frame(&mut self, frame: &Frame) -> Result<(), GraphicsError> {
        self.framebuffer.present_frame(self.egl.gl(), frame);

        self.egl
            .instance()
            .swap_buffers(self.egl.display(), self.egl.surface())?;

        let buffer_object = unsafe { self.gbm.surface().lock_front_buffer() }?;
        let bpp = buffer_object.bpp();
        let drm_fb = self.drm.gpu().add_framebuffer(&buffer_object, bpp, bpp)?;

        self.drm
            .gpu()
            .page_flip(self.drm.crtc().handle(), drm_fb, PageFlipFlags::EVENT, None)?;
        let _events = self.drm.gpu().receive_events()?;

        self.drm.gpu().destroy_framebuffer(self.drm_fb)?;

        self.buffer_object = buffer_object;
        self.drm_fb = drm_fb;

        std::thread::sleep(Self::FRAME_DURATION.saturating_sub(self.frame_start.elapsed()));
        self.frame_start = Instant::now();
        self.update_fps();

        Ok(())
    }

    #[must_use]
    pub fn fps(&self) -> u32 {
        self.fps
    }

    fn update_fps(&mut self) {
        self.fps_frames = self.fps_frames.saturating_add(1);
        let elapsed = self.fps_timer.elapsed();
        if elapsed >= Duration::from_secs(1) {
            self.fps = ((self.fps_frames as f64) / elapsed.as_secs_f64()).round() as u32;
            self.fps_frames = 0;
            self.fps_timer = Instant::now();
        }
    }
}

impl Drop for Graphics {
    fn drop(&mut self) {
        if let Err(e) = self.drm.gpu().destroy_framebuffer(self.drm_fb) {
            log::error!("failed to destroy framebuffer on Graphics drop: {e}");
        }
        GRAPHICS_LOADED.store(false, Ordering::Relaxed);
    }
}
