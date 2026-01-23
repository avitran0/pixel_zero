use std::sync::Arc;

use gbm::{BufferObjectFlags, Device, Surface};
use glam::UVec2;

use crate::graphics::drm::{Drm, Gpu};

pub struct Gbm {
    size: UVec2,
    device: Device<Arc<Gpu>>,
    surface: Surface<()>,
}

impl Gbm {
    pub fn load(drm: &Drm) -> anyhow::Result<Self> {
        let size = drm.size();
        let device = Device::new(drm.gpu_arc())?;
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

    pub fn device(&self) -> &Device<Arc<Gpu>> {
        &self.device
    }

    pub fn surface(&self) -> &Surface<()> {
        &self.surface
    }

    pub fn size(&self) -> UVec2 {
        self.size
    }

    pub fn init_surface(&mut self, format: gbm::Format) -> anyhow::Result<()> {
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
