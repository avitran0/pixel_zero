use pixel_zero::{
    glam::ivec2,
    graphics::{Font, Frame, Graphics},
    input::{Button, Input},
};

pub struct Game {
    graphics: Graphics,
    input: Input,
    font: Font,
    exit: bool,
}

impl Game {
    pub fn new() -> Self {
        let graphics = Graphics::load().unwrap();
        let font = graphics
            .load_font_binary(include_bytes!("../assets/cozette.psf"))
            .unwrap();

        Self {
            graphics,
            input: Input::default(),
            font,
            exit: false,
        }
    }

    pub fn run(&mut self) {
        while !self.exit {
            self.input.update();
            if self.input.just_pressed(Button::Start) {
                break;
            }

            let mut frame = Frame::default();

            frame.draw_text(&self.font, "text here", ivec2(0, 0));
            frame.draw_text(
                &self.font,
                &format!("FPS: {}", self.graphics.fps()),
                ivec2(0, self.font.glyph_size().y.cast_signed()),
            );

            self.graphics.present_frame(&frame).unwrap();
        }
    }
}
