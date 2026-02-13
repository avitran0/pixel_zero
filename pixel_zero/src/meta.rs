pub use elf_macros::embed_metadata;
use thiserror::Error;

use std::{
    io::{Cursor, Seek as _, SeekFrom},
    path::{Path, PathBuf},
    string::FromUtf8Error,
};

use goblin::elf::Elf;

use crate::io::ReadBytes as _;

/// Metadata about a single game.
pub struct GameInfo {
    pub name: String,
    pub version: u32,
    pub path: PathBuf,
}

#[derive(Debug, Error)]
pub enum ReadMetadataError {
    #[error("I/O error: {0}")]
    IO(#[from] std::io::Error),
    #[error("ELF parse error: {0}")]
    ElfParse(#[from] goblin::error::Error),
    #[error("`gamemeta` section not found in ELF")]
    SectionNotFound,
    #[error("Invalid GameInfo magic: {0:?}")]
    InvalidMagic(Vec<u8>),
    #[error("Invalid string: {0}")]
    Utf8(#[from] FromUtf8Error),
}

/// Tries to read the Metadata for an executable.
///
/// # Errors
///
/// Fails if it cannot open or parse the executable, the game info section is not present, or malformed.
pub fn read_metadata(path: impl AsRef<Path>) -> Result<GameInfo, ReadMetadataError> {
    let bytes = std::fs::read(&path)?;
    let elf = Elf::parse(&bytes)?;

    let section = elf
        .section_headers
        .iter()
        .find(|sh| elf.shdr_strtab.get_at(sh.sh_name) == Some(".gamemeta"))
        .ok_or(ReadMetadataError::SectionNotFound)?;

    let mut reader = Cursor::new(bytes.as_slice());
    reader.seek(SeekFrom::Start(section.sh_offset))?;

    let magic = reader.read_bytes(8)?;
    if magic != b"gamemeta" {
        return Err(ReadMetadataError::InvalidMagic(magic));
    }

    let version = reader.read_u32()?;
    let name_len = reader.read_u32()?;

    let name = reader.read_bytes(name_len as usize)?;

    let name = String::from_utf8(name)?;
    let path = path.as_ref().to_path_buf();

    Ok(GameInfo {
        name,
        version,
        path,
    })
}
