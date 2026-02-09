use std::sync::atomic::{AtomicBool, Ordering};

use ::drm::control::{self, Device as _, PageFlipFlags, framebuffer as drmfb};
use ::gbm::BufferObject;
use glam::UVec2;

use crate::graphics::{
    color::Color, drm::Drm, egl::Egl, framebuffer::Framebuffer, gbm::Gbm, sprite::Sprite,
};

pub mod color;
mod drm;
mod egl;
mod frame;
mod framebuffer;
mod gbm;
mod quad;
mod shader;
pub mod sprite;
mod texture;

pub struct Graphics {
    drm: Drm,
    gbm: Gbm,
    egl: Egl,

    drm_fb: drmfb::Handle,
    buffer_object: BufferObject<()>,

    framebuffer: Framebuffer,
}

static LOADED: AtomicBool = AtomicBool::new(false);
impl Graphics {
    pub fn load() -> anyhow::Result<Self> {
        if LOADED.swap(true, Ordering::Relaxed) {
            return Err(anyhow::anyhow!("graphics already loaded"));
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

        Ok(Self {
            drm,
            gbm,
            egl,
            drm_fb,
            buffer_object,
            framebuffer,
        })
    }

    pub fn clear(&self, color: Color) {
        self.framebuffer.clear(color);
    }

    pub fn draw_sprite(&self, sprite: &Sprite, position: UVec2) {
        self.framebuffer.draw_sprite(sprite, position);
    }

    pub fn present(&mut self) -> anyhow::Result<()> {
        self.framebuffer.present();

        self.egl
            .instance()
            .swap_buffers(self.egl.display(), self.egl.surface())?;

        let buffer_object = unsafe { self.gbm.surface().lock_front_buffer() }?;
        let bpp = buffer_object.bpp();
        let drm_fb = self.drm.gpu().add_framebuffer(&buffer_object, bpp, bpp)?;

        self.drm
            .gpu()
            .page_flip(self.drm.crtc().handle(), drm_fb, PageFlipFlags::EVENT, None)?;
        let events = self.drm.gpu().receive_events()?;
        for event in events {
            if let control::Event::PageFlip(_event) = event {
                // todo
            }
        }

        self.drm.gpu().destroy_framebuffer(self.drm_fb)?;

        self.buffer_object = buffer_object;
        self.drm_fb = drm_fb;

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
