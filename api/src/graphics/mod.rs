use std::sync::atomic::{AtomicBool, Ordering};

use ::drm::control::{self, Device as _, PageFlipFlags, framebuffer};
use ::gbm::BufferObject;

use crate::graphics::{drm::Drm, egl::Egl, gbm::Gbm};

pub mod color;
mod drm;
mod egl;
mod gbm;

pub struct GraphicsContext {
    drm: Drm,
    gbm: Gbm,
    egl: Egl,

    framebuffer: framebuffer::Handle,
    buffer_object: BufferObject<()>,
}

static LOADED: AtomicBool = AtomicBool::new(false);
impl GraphicsContext {
    pub fn load() -> anyhow::Result<Self> {
        if LOADED.swap(true, Ordering::Relaxed) {
            return Err(anyhow::anyhow!("GraphicsContext already loaded"));
        }

        let drm = Drm::load()?;
        let gbm = Gbm::load(&drm)?;
        let egl = Egl::load(&gbm, drm.size())?;

        let buffer_object = unsafe { gbm.surface().lock_front_buffer() }?;
        let bpp = buffer_object.bpp();
        let framebuffer = drm.gpu().add_framebuffer(&buffer_object, bpp, bpp)?;
        drm.gpu().set_crtc(
            drm.crtc().handle(),
            Some(framebuffer),
            (0, 0),
            &[drm.connector().handle()],
            Some(*drm.mode()),
        )?;

        Ok(Self {
            drm,
            gbm,
            egl,
            framebuffer,
            buffer_object,
        })
    }

    pub fn clear(&self) {
        unsafe {
            gl::ClearColor(0.0, 0.0, 0.0, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT)
        };
    }

    pub fn present(&mut self) -> anyhow::Result<()> {
        self.egl
            .instance()
            .swap_buffers(self.egl.display(), self.egl.surface())?;

        let buffer_object = unsafe { self.gbm.surface().lock_front_buffer() }?;
        let bpp = buffer_object.bpp();
        let framebuffer = self.drm.gpu().add_framebuffer(&buffer_object, bpp, bpp)?;

        self.drm.gpu().page_flip(
            self.drm.crtc().handle(),
            framebuffer,
            PageFlipFlags::EVENT,
            None,
        )?;
        let events = self.drm.gpu().receive_events()?;
        for event in events {
            if let control::Event::PageFlip(event) = event {
                // todo
            }
        }

        self.drm.gpu().destroy_framebuffer(self.framebuffer)?;

        self.buffer_object = buffer_object;
        self.framebuffer = framebuffer;

        Ok(())
    }
}
