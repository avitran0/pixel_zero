use pixel_zero::{Font, Frame, Graphics, input::Input, ivec2, meta::embed_metadata};

embed_metadata!(name: "Game", version: 1);

fn main() {
    let mut graphics = Graphics::load().unwrap();
    let mut input = Input::default();
    let font = Font::load_binary(include_bytes!("../assets/cozette.psf")).unwrap();

    loop {
        input.update();
        let mut frame = Frame::default();

        frame.draw_text(&font, "text here", ivec2(0, 0));

        graphics.present_frame(&frame).unwrap();
    }
}
