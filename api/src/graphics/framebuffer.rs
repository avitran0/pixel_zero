use glam::uvec2;

use crate::{
    HEIGHT, WIDTH,
    graphics::{shader::Shader, texture::Texture},
};

pub(crate) struct Framebuffer {
    framebuffer: u32,
    fbo_texture: Texture,
    sprite_shader: Shader,
    screen_shader: Shader,
}

impl Framebuffer {
    pub fn load() -> anyhow::Result<Self> {
        let mut framebuffer = 0;
        unsafe {
            gl::GenFramebuffers(1, &mut framebuffer);
            gl::BindFramebuffer(gl::FRAMEBUFFER, framebuffer);
        }

        let texture = Texture::empty(uvec2(WIDTH, HEIGHT))?;

        unsafe {
            gl::FramebufferTexture2D(gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0, gl::TEXTURE_2D, texture.handle(), 0);
        }

        Err(anyhow::anyhow!(""))
    }
}
