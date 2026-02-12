use std::fs::File;

use glam::ivec2;
use pixel_zero::{
    graphics::{font::Font, Graphics},
    input::Input,
    io::ReadBytes as _,
    meta::{read_metadata, GameInfo},
};

use crate::screen::Screen;

pub struct GameMenu {
    games: Vec<GameInfo>,
    font: Font,
}

impl GameMenu {
    pub fn new() -> Self {
        let exe_dir = std::env::current_exe().unwrap();
        let dir = exe_dir.parent().unwrap();
        let games: Vec<GameInfo> = std::fs::read_dir(dir)
            .unwrap()
            .filter_map(|entry| {
                let entry = entry.ok()?;
                let mut file = File::open(entry.path()).ok()?;
                let magic = file.read_u32().ok()?;
                if magic == 0x7F454C46 || magic == 0x464C457F {
                    Some(entry.path())
                } else {
                    None
                }
            })
            .filter_map(read_metadata)
            .collect();

        log::info!(
            "found {} game{}",
            games.len(),
            if games.len() == 1 { "" } else { "s" }
        );

        Self {
            games,
            font: Font::load_bin(include_bytes!("cozette.psf")).unwrap(),
        }
    }
}

impl Screen for GameMenu {
    fn update(&mut self, input: &Input) {}

    fn render(&self, graphics: &Graphics) {
        let mut offset = 0;
        for game in &self.games {
            graphics.draw_text(&self.font, &game.name, ivec2(0, offset));
            offset += self.font.glyph_size().y.cast_signed();
        }
    }
}
