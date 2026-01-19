use std::{
    fs::File,
    os::{
        fd::{AsFd, BorrowedFd},
        unix::fs::FileTypeExt,
    },
    sync::Arc,
};

use api::graphics::Graphics;
use drm::{
    Device as DrmDevice,
    control::{Device as ControlDevice, Mode, ModeTypeFlags, connector, crtc},
};
use gbm::{AsRaw as _, BufferObjectFlags, Device as GbmDevice, Surface as GbmSurface};
use glam::UVec2;
use khronos_egl as egl;

pub struct GraphicsContext {
    drm: Drm,
    gbm: Gbm,
}

impl GraphicsContext {}

impl Graphics for GraphicsContext {
    fn clear(&self, color: api::graphics::Color) {}
}

struct Drm {
    gpu: Arc<Gpu>,
    connector: connector::Info,
    mode: Mode,
    crtc: crtc::Info,
}

impl Drm {
    fn load() -> anyhow::Result<Self> {
        let gpu = Gpu::open()?;

        let resources = gpu.resource_handles()?;

        let Some(connector) = resources
            .connectors()
            .iter()
            .flat_map(|handle| gpu.get_connector(*handle, true))
            .find(|connector| connector.state() == connector::State::Connected)
        else {
            return Err(anyhow::anyhow!("No connected connectors found"));
        };

        let mode = *connector
            .modes()
            .iter()
            .find(|mode| mode.mode_type().contains(ModeTypeFlags::PREFERRED))
            .unwrap_or_else(|| &connector.modes()[0]);

        let Some(crtc) = connector
            .encoders()
            .iter()
            .flat_map(|handle| gpu.get_encoder(*handle))
            .flat_map(|encoder| encoder.crtc())
            .flat_map(|crtc| gpu.get_crtc(crtc))
            .next()
        else {
            return Err(anyhow::anyhow!("No suitable CRTC found"));
        };

        Ok(Self {
            gpu: Arc::new(gpu),
            connector,
            mode,
            crtc,
        })
    }

    fn size(&self) -> UVec2 {
        UVec2 {
            x: self.mode.size().0 as u32,
            y: self.mode.size().1 as u32,
        }
    }

    fn gpu(&self) -> Arc<Gpu> {
        self.gpu.clone()
    }
}

struct Gbm {
    device: GbmDevice<Arc<Gpu>>,
    surface: GbmSurface<()>,
}

impl Gbm {
    pub fn load(drm: &Drm) -> anyhow::Result<Self> {
        let size = drm.size();
        let device = GbmDevice::new(drm.gpu())?;
        let surface = device.create_surface(
            size.x,
            size.y,
            gbm::Format::Xrgb8888,
            BufferObjectFlags::SCANOUT | BufferObjectFlags::RENDERING,
        )?;
        Ok(Self { device, surface })
    }
}

struct Egl {}

impl Egl {
    pub fn load(gbm: &Gbm) -> anyhow::Result<Self> {
        let egl = egl::Instance::new(egl::Static);
        let display = unsafe { egl.get_display(gbm.device.as_raw() as *mut _) }
            .ok_or(|| anyhow::anyhow!("No EGL Display found"))?;
        let egl_version = egl.initialize(display)?;
        egl.bind_api(egl::OPENGL_ES_API);

        let config_attributes = [
            egl::RED_SIZE,
            8,
            egl::GREEN_SIZE,
            8,
            egl::BLUE_SIZE,
            8,
            egl::RENDERABLE_TYPE,
            egl::OPENGL_ES2_BIT,
            egl::SURFACE_TYPE,
            egl::WINDOW_BIT,
            egl::NONE,
        ];

        
    }
}

struct Gpu {
    file: File,
}

impl Gpu {
    pub fn open() -> std::io::Result<Self> {
        let dir = std::fs::read_dir("/dev/dri")?;
        for file in dir {
            let file = file?;
            if !file.file_type()?.is_char_device() {
                continue;
            }
            let name_osstr = file.file_name();
            let Some(name) = name_osstr.to_str() else {
                continue;
            };
            if !name.starts_with("card") {
                continue;
            }
            let file = File::options().write(true).read(true).open(file.path())?;
            return Ok(Self { file });
        }
        Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "No valid DRM device found",
        ))
    }
}

impl AsFd for Gpu {
    fn as_fd(&self) -> BorrowedFd<'_> {
        self.file.as_fd()
    }
}

impl DrmDevice for Gpu {}
impl ControlDevice for Gpu {}

pub fn resolutions() -> std::io::Result<Vec<String>> {
    let card = Gpu::open()?;
    let resources = card.resource_handles()?;
    let Some(connector) = resources
        .connectors()
        .iter()
        .flat_map(|handle| card.get_connector(*handle, true))
        .find(|connector| connector.state() == connector::State::Connected)
    else {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "No connected connectors found",
        ));
    };
    Ok(connector
        .modes()
        .iter()
        .map(|mode| format!("{}x{} {}hz", mode.size().0, mode.size().1, mode.vrefresh()))
        .collect())
}
