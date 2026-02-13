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
    control::{Device as ControlDevice, Mode, ModeTypeFlags, connector, crtc, framebuffer},
};
use glam::UVec2;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DrmError {
    #[error("I/O error: {0}")]
    IO(#[from] std::io::Error),
    #[error("No Connectors found")]
    NoConnectors,
    #[error("No suitable CRTC found")]
    NoCRTC,
}

struct OriginalState {
    crtc: crtc::Info,
    framebuffer: Option<framebuffer::Handle>,
    connectors: Vec<connector::Handle>,
    mode: Option<Mode>,
}

pub(crate) struct Drm {
    gpu: Arc<Gpu>,
    connector: connector::Info,
    mode: Mode,
    crtc: crtc::Info,
    original_state: Option<OriginalState>,
}

impl Drm {
    pub(crate) fn load() -> Result<Self, DrmError> {
        let gpu = Gpu::open()?;

        let resources = gpu.resource_handles()?;

        let Some(connector) = resources
            .connectors()
            .iter()
            .flat_map(|handle| gpu.get_connector(*handle, true))
            .find(|connector| connector.state() == connector::State::Connected)
        else {
            return Err(DrmError::NoConnectors);
        };

        let original_crtc = connector
            .current_encoder()
            .and_then(|e| gpu.get_encoder(e).ok())
            .and_then(|e| e.crtc());

        let original_state = if let Some(crtc) = original_crtc {
            let crtc_info = gpu.get_crtc(crtc)?;
            let connectors: Vec<_> = resources
                .connectors()
                .iter()
                .filter_map(|&conn_handle| gpu.get_connector(conn_handle, false).ok())
                .filter(|conn| {
                    conn.current_encoder()
                        .and_then(|enc_handle| gpu.get_encoder(enc_handle).ok())
                        .and_then(|enc| enc.crtc())
                        == Some(crtc)
                })
                .map(|conn| conn.handle())
                .collect();

            Some(OriginalState {
                crtc: crtc_info,
                framebuffer: crtc_info.framebuffer(),
                connectors,
                mode: crtc_info.mode(),
            })
        } else {
            None
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
            .filter_map(|encoder| encoder.crtc())
            .flat_map(|crtc| gpu.get_crtc(crtc))
            .next()
        else {
            return Err(DrmError::NoCRTC);
        };

        Ok(Self {
            gpu: Arc::new(gpu),
            connector,
            mode,
            crtc,
            original_state,
        })
    }

    pub(crate) fn size(&self) -> UVec2 {
        UVec2 {
            x: u32::from(self.mode.size().0),
            y: u32::from(self.mode.size().1),
        }
    }

    pub(crate) fn gpu(&self) -> &Gpu {
        &self.gpu
    }

    pub(crate) fn gpu_arc(&self) -> Arc<Gpu> {
        self.gpu.clone()
    }

    pub(crate) fn connector(&self) -> &connector::Info {
        &self.connector
    }

    pub(crate) fn mode(&self) -> &Mode {
        &self.mode
    }

    pub(crate) fn crtc(&self) -> &crtc::Info {
        &self.crtc
    }
}

impl Drop for Drm {
    fn drop(&mut self) {
        if let Some(state) = &self.original_state {
            let _ = self.gpu.set_crtc(
                state.crtc.handle(),
                state.framebuffer,
                state.crtc.position(),
                &state.connectors,
                state.mode,
            );
        }
    }
}

pub(crate) struct Gpu {
    file: File,
}

impl Gpu {
    pub(crate) fn open() -> std::io::Result<Self> {
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
