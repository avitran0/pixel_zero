pub use elf_macros::*;

use std::{
    io::{Cursor, Seek as _, SeekFrom},
    path::{Path, PathBuf},
};

use goblin::elf::Elf;

use crate::io::ReadBytes as _;

pub struct GameInfo {
    pub name: String,
    pub version: u32,
    pub path: PathBuf,
}

pub fn read_metadata(path: impl AsRef<Path>) -> Option<GameInfo> {
    let bytes = std::fs::read(&path).ok()?;
    let elf = Elf::parse(&bytes).ok()?;

    let section = elf
        .section_headers
        .iter()
        .find(|sh| elf.shdr_strtab.get_at(sh.sh_name) == Some(".gamemeta"))?;

    let mut reader = Cursor::new(bytes.as_slice());
    reader.seek(SeekFrom::Start(section.sh_offset)).ok()?;

    let magic = reader.read_bytes(8).ok()?;
    if magic != b"gamemeta" {
        return None;
    }

    let version = reader.read_u32().ok()?;
    let name_len = reader.read_u32().ok()?;

    let name = reader.read_bytes(name_len as usize).ok()?;

    let name = String::from_utf8(name).ok()?;
    let path = path.as_ref().to_path_buf();

    Some(GameInfo {
        name,
        version,
        path,
    })
}
