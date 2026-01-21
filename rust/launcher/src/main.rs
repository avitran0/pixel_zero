/*#[link(name = "EGL")]
#[link(name = "GLESv2")]
unsafe extern "C" {}*/

mod graphics;

fn main() {
    dbg!(graphics::resolutions().unwrap());
}
