use pixel_zero::{Font, Graphics, meta::embed_metadata, input::Input, ivec2};

embed_metadata!(name: "Game", version: 1);

fn main() {
    let mut graphics = Graphics::load().unwrap();
    let mut input = Input::default();
    let font = Font::load_bin(include_bytes!("../assets/cozette.psf")).unwrap();

    loop {
        input.update();

        graphics.draw_text(&font, "text here", ivec2(0, 0));

        graphics.present().unwrap();
    }
}
