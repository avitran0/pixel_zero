use glam::IVec2;

use crate::graphics::{Color, Font, Sprite};

pub struct Frame {
    draw_commands: Vec<DrawCommand>,
    clear_color: Color,
}

impl Frame {
    pub fn draw_sprite(&mut self, sprite: &Sprite, position: IVec2) {
        self.draw_commands.push(DrawCommand::Sprite {
            sprite: sprite.clone(),
            position,
        })
    }

    pub fn draw_text(&mut self, font: &Font, text: &str, position: IVec2) {
        self.draw_commands.push(DrawCommand::Text {
            font: font.clone(),
            text: text.to_owned(),
            position,
        })
    }

    pub fn set_clear_color(&mut self, color: Color) {
        self.clear_color = color
    }

    pub(crate) fn clear_color(&self) -> Color {
        self.clear_color
    }

    pub(crate) fn commands(&self) -> &[DrawCommand] {
        &self.draw_commands
    }
}

impl Default for Frame {
    fn default() -> Self {
        Self {
            draw_commands: Vec::new(),
            clear_color: Color::BLACK,
        }
    }
}

pub(crate) enum DrawCommand {
    Sprite {
        sprite: Sprite,
        position: IVec2,
    },
    Text {
        font: Font,
        text: String,
        position: IVec2,
    },
}
