use std::{
    fs::File,
    os::{
        fd::{AsFd, BorrowedFd},
        unix::fs::FileTypeExt,
    },
};

use drm::{
    Device as DrmDevice,
    control::{Device as ControlDevice, Mode, ModeTypeFlags, connector, crtc},
};

pub struct Graphics {
    drm: Drm,
}

impl Graphics {}

pub struct Drm {
    gpu: Gpu,
    connector: connector::Info,
    mode: Mode,
    crtc: crtc::Info,
}

impl Drm {
    pub fn load() -> std::io::Result<Self> {
        let gpu = Gpu::open()?;

        let resources = gpu.resource_handles()?;

        let Some(connector) = resources
            .connectors()
            .iter()
            .flat_map(|handle| gpu.get_connector(*handle, true))
            .find(|connector| connector.state() == connector::State::Connected)
        else {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "No connected connectors found",
            ));
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
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "No suitable CRTC found",
            ));
        };

        Ok(Self {
            gpu,
            connector,
            mode,
            crtc,
        })
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
