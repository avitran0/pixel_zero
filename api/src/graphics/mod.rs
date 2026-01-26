use std::sync::atomic::{AtomicBool, Ordering};

use ::drm::control::{self, Device as _, PageFlipFlags, framebuffer as drmfb};
use ::gbm::BufferObject;

use crate::graphics::{
    color::{Color, ColorF32}, drm::Drm, egl::Egl, framebuffer::Framebuffer, gbm::Gbm
};

pub mod color;
mod drm;
mod egl;
mod framebuffer;
mod gbm;
mod shader;
mod sprite;
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

        let framebuffer = Framebuffer::load()?;

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
        let color = ColorF32::from(color);
        unsafe {
            gl::ClearColor(color.r(), color.g(), color.b(), color.a());
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT)
        };
    }

    pub fn present(&mut self) -> anyhow::Result<()> {
        self.egl
            .instance()
            .swap_buffers(self.egl.display(), self.egl.surface())?;

        let buffer_object = unsafe { self.gbm.surface().lock_front_buffer() }?;
        let bpp = buffer_object.bpp();
        let drm_fb = self.drm.gpu().add_framebuffer(&buffer_object, bpp, bpp)?;

        self.drm.gpu().page_flip(
            self.drm.crtc().handle(),
            drm_fb,
            PageFlipFlags::EVENT,
            None,
        )?;
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
}
