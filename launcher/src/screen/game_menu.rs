use std::fs::File;

use pixel_zero::{
    glam::{ivec2, uvec2},
    graphics::{Color, Font, Frame, Graphics},
    input::{Button, Input},
    io::ReadBytes as _,
    meta::{GameInfo, read_metadata},
};

use crate::screen::Screen;

pub struct GameMenu {
    games: Vec<GameInfo>,
    button_state: [bool; Button::BUTTON_COUNT],
}

impl GameMenu {
    pub fn init(_graphics: &Graphics) -> Self {
        let exe_dir = std::env::current_exe().unwrap();
        let dir = exe_dir.parent().unwrap();
        let games: Vec<GameInfo> = std::fs::read_dir(dir)
            .unwrap()
            .filter_map(|entry| {
                let entry = entry.ok()?;
                let mut file = File::open(entry.path()).ok()?;
                let magic = file.read_u32().ok()?;
                if magic == 0x7F45_4C46 || magic == 0x464C_457F {
                    Some(entry.path())
                } else {
                    None
                }
            })
            .filter_map(|entry| read_metadata(entry).ok())
            .collect();

        log::info!(
            "found {} game{}",
            games.len(),
            if games.len() == 1 { "" } else { "s" }
        );

        let button_state = [false; Button::BUTTON_COUNT];

        Self {
            games,
            button_state,
        }
    }
}

impl Screen for GameMenu {
    fn update(&mut self, input: &Input) {
        self.button_state = *input.state();
    }

    fn render(&self, frame: &mut Frame, font: &Font) {
        let mut offset = 0;
        for game in &self.games {
            frame.draw_text(font, &game.name, ivec2(200, offset));
            offset += font.glyph_size().y.cast_signed();
        }

        for (index, button) in self.button_state.iter().enumerate() {
            if *button {
                let button = Button::from_usize(index).unwrap();
                frame.draw_text(font, &format!("{button:?}"), ivec2(200, offset));
                offset += font.glyph_size().y.cast_signed();
            }
        }

        let mut offset = 0;
        for i in (0..15).rev() {
            frame.draw_line(ivec2(offset, 50), ivec2(offset + i, 50 + i), Color::WHITE);
            offset += 4;
        }

        offset = 0;
        for i in (0..15).rev() {
            frame.draw_rect_outline(
                ivec2(offset, 0),
                uvec2(i as u32 + 1, i as u32 + 1),
                Color::WHITE,
            );
            frame.draw_rect(
                ivec2(offset, 20),
                uvec2(i as u32 + 1, i as u32 + 1),
                Color::WHITE,
            );
            offset += i + 3;
        }

        let mut line_offset = 0;
        for i in (0..15).rev() {
            frame.draw_line(
                ivec2(line_offset, 70),
                ivec2(line_offset, 70 + i),
                Color::WHITE,
            );
            frame.draw_line(
                ivec2(0, 90 + line_offset),
                ivec2(i, 90 + line_offset),
                Color::WHITE,
            );
            line_offset += 2;
        }
    }
}
