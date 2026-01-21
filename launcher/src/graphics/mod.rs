use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use ::drm::control::{self, Device as _, PageFlipFlags, framebuffer};
use ::gbm::{
    AsRaw as _, BufferObject, BufferObjectFlags, Device as GbmDevice, Surface as GbmSurface,
};
use api::graphics::Graphics;
use egui::RawInput;
use glam::UVec2;

use crate::graphics::{drm::Drm, egl::Egl, gbm::Gbm};

mod drm;
mod egl;
mod gbm;

pub struct GraphicsContext {
    drm: Drm,
    gbm: Gbm,
    egl: Egl,

    framebuffer: Option<framebuffer::Handle>,
    buffer_object: Option<BufferObject<()>>,

    raw_input: egui::RawInput,
    context: egui::Context,
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

        let raw_input = RawInput::default();
        let context = egui::Context::default();

        Ok(Self {
            drm,
            gbm,
            egl,
            framebuffer: Some(framebuffer),
            buffer_object: Some(buffer_object),
            raw_input,
            context,
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

        if let Some(framebuffer) = &self.framebuffer {
            self.drm.gpu().destroy_framebuffer(*framebuffer)?;
        }

        self.buffer_object = Some(buffer_object);
        self.framebuffer = Some(framebuffer);

        Ok(())
    }
}

impl Graphics for GraphicsContext {
    fn clear(&self, color: api::graphics::Color) {}
}
