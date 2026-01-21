use std::{
    fs::File,
    os::{
        fd::{AsFd, BorrowedFd},
        unix::fs::FileTypeExt as _,
    },
    sync::Arc,
};

use drm::{
    Device as DrmDevice,
    control::{Device as ControlDevice, Mode, ModeTypeFlags, connector, crtc},
};
use glam::UVec2;

pub struct Drm {
    gpu: Arc<Gpu>,
    connector: connector::Info,
    mode: Mode,
    crtc: crtc::Info,
}

impl Drm {
    pub fn load() -> anyhow::Result<Self> {
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

    pub fn size(&self) -> UVec2 {
        UVec2 {
            x: self.mode.size().0 as u32,
            y: self.mode.size().1 as u32,
        }
    }

    pub fn gpu(&self) -> &Gpu {
        &self.gpu
    }

    pub fn gpu_arc(&self) -> Arc<Gpu> {
        self.gpu.clone()
    }

    pub fn connector(&self) -> &connector::Info {
        &self.connector
    }

    pub fn mode(&self) -> &Mode {
        &self.mode
    }

    pub fn crtc(&self) -> &crtc::Info {
        &self.crtc
    }
}

pub struct Gpu {
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
