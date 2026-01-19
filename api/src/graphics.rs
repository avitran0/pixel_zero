use std::{
    fs::File,
    os::{
        fd::{AsFd, BorrowedFd},
        unix::fs::FileTypeExt,
    },
};

use drm::{
    Device as DrmDevice,
    control::{Device as ControlDevice, connector},
};

pub struct Graphics {}

struct Card {
    file: File,
}

impl Card {
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

impl AsFd for Card {
    fn as_fd(&self) -> BorrowedFd<'_> {
        self.file.as_fd()
    }
}

impl DrmDevice for Card {}
impl ControlDevice for Card {}

pub fn resolutions() -> std::io::Result<Vec<String>> {
    let card = Card::open()?;
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
