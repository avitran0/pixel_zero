use std::{
    sync::atomic::{AtomicBool, Ordering},
    time::{Duration, Instant},
};

use ::drm::control::{Device as _, PageFlipFlags, framebuffer as drmfb};
use ::gbm::{BufferObject, FrontBufferError};
use thiserror::Error;

use crate::{
    Frame,
    graphics::{
        drm::{Drm, DrmError},
        egl::Egl,
        framebuffer::Framebuffer,
        gbm::Gbm,
        shader::ShaderError,
    },
};

pub mod color;
mod drm;
mod egl;
pub mod font;
pub mod frame;
mod framebuffer;
mod gbm;
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
    #[error("Shader Error: {0}")]
    Shader(#[from] ShaderError),
    #[error("Front Buffer Error: {0}")]
    FrontBuffer(#[from] FrontBufferError),
    #[error("Graphics is already loaded")]
    AlreadyLoaded,
}

pub struct Graphics {
    // this needs to be first to be dropped first
    framebuffer: Framebuffer,
    frame_start: Instant,

    drm_fb: drmfb::Handle,
    buffer_object: BufferObject<()>,

    drm: Drm,
    gbm: Gbm,
    egl: Egl,
}

pub(crate) static GRAPHICS_LOADED: AtomicBool = AtomicBool::new(false);
impl Graphics {
    pub fn load() -> Result<Self, GraphicsError> {
        if GRAPHICS_LOADED.swap(true, Ordering::Relaxed) {
            return Err(GraphicsError::AlreadyLoaded);
        }

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

        let framebuffer = Framebuffer::load(drm.size())?;
        let frame_start = Instant::now();

        Ok(Self {
            drm,
            gbm,
            egl,
            drm_fb,
            buffer_object,
            framebuffer,
            frame_start,
        })
    }

    const FRAME_DURATION: Duration = Duration::from_micros(16667);
    pub fn present_frame(&mut self, frame: &Frame) -> Result<(), GraphicsError> {
        self.framebuffer.present_frame(frame);

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

        Ok(())
    }

    pub fn check_error(&self) {
        loop {
            let error = unsafe { gl::GetError() };
            if error == gl::NO_ERROR {
                break;
            }

            let err_str = match error {
                gl::INVALID_ENUM => "Invalid Enum",
                gl::INVALID_VALUE => "Invalid Value",
                gl::INVALID_OPERATION => "Invalid Operation",
                gl::INVALID_FRAMEBUFFER_OPERATION => "Invalid Framebuffer Operation",
                gl::OUT_OF_MEMORY => "Out Of Memory",
                gl::STACK_UNDERFLOW => "Stack Underflow",
                gl::STACK_OVERFLOW => "Stack Overflow",
                _ => "?",
            };

            log::error!("opengl error {error}: {err_str}");
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
