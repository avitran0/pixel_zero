/*#[link(name = "EGL")]
#[link(name = "GLESv2")]
unsafe extern "C" {}*/

use crate::platform::GraphicsContext;

mod platform;

fn main() {
    let mut graphics = GraphicsContext::load().unwrap();
    graphics.clear();
    graphics.present().unwrap();
}
