use std::sync::Arc;

use gbm::{BufferObjectFlags, Device, Surface};

use crate::graphics::drm::{Drm, Gpu};

pub struct Gbm {
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
        Ok(Self { device, surface })
    }

    pub fn device(&self) -> &Device<Arc<Gpu>> {
        &self.device
    }

    pub fn surface(&self) -> &Surface<()> {
        &self.surface
    }
}
