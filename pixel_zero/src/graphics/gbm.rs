use std::sync::Arc;

use gbm::{BufferObjectFlags, Device, Surface};
use glam::UVec2;

use crate::graphics::drm::{Drm, Gpu};

pub(crate) struct Gbm {
    size: UVec2,
    device: Device<Arc<Gpu>>,
    surface: Surface<()>,
}

impl Gbm {
    pub(crate) fn load(drm: &Drm) -> std::io::Result<Self> {
        let size = drm.size();
        let device = Device::new(drm.gpu_arc())?;
        log::info!("gbm backend: {}", device.backend_name());
        let surface = device.create_surface(
            size.x,
            size.y,
            gbm::Format::Xrgb8888,
            BufferObjectFlags::SCANOUT | BufferObjectFlags::RENDERING,
        )?;
        Ok(Self {
            size,
            device,
            surface,
        })
    }

    pub(crate) fn device(&self) -> &Device<Arc<Gpu>> {
        &self.device
    }

    pub(crate) fn surface(&self) -> &Surface<()> {
        &self.surface
    }

    pub(crate) fn size(&self) -> UVec2 {
        self.size
    }

    pub(crate) fn init_surface(&mut self, format: gbm::Format) -> std::io::Result<()> {
        let surface = self.device.create_surface(
            self.size.x,
            self.size.y,
            format,
            BufferObjectFlags::SCANOUT | BufferObjectFlags::RENDERING,
        )?;
        self.surface = surface;
        Ok(())
    }
}
