use glam::{IVec2, UVec2};

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
        });
    }

    pub fn draw_text(&mut self, font: &Font, text: &str, position: IVec2) {
        self.draw_commands.push(DrawCommand::Text {
            font: font.clone(),
            text: text.to_owned(),
            position,
        });
    }

    pub fn draw_rect(&mut self, position: IVec2, size: UVec2, color: Color) {
        self.draw_commands.push(DrawCommand::Rect {
            position,
            size,
            color,
            filled: true,
        });
    }

    pub fn draw_rect_outline(&mut self, position: IVec2, size: UVec2, color: Color) {
        self.draw_commands.push(DrawCommand::Rect {
            position,
            size,
            color,
            filled: false,
        });
    }

    pub fn draw_line(&mut self, start: IVec2, end: IVec2, color: Color) {
        self.draw_commands.push(DrawCommand::Line {
            start,
            end,
            width: 1,
            color,
        });
    }

    pub fn set_clear_color(&mut self, color: Color) {
        self.clear_color = color;
    }

    pub(crate) fn add_command(&mut self, command: DrawCommand) {
        self.draw_commands.push(command);
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
    Line {
        start: IVec2,
        end: IVec2,
        width: u32,
        color: Color,
    },
    Rect {
        position: IVec2,
        size: UVec2,
        color: Color,
        filled: bool,
    },
}
