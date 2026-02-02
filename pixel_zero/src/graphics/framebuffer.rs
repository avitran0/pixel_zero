use glam::uvec2;

use crate::{
    HEIGHT, WIDTH,
    graphics::{shader::Shader, texture::Texture},
};

pub(crate) struct Framebuffer {
    framebuffer: u32,
    texture: Texture,
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
            gl::FramebufferTexture2D(
                gl::FRAMEBUFFER,
                gl::COLOR_ATTACHMENT0,
                gl::TEXTURE_2D,
                texture.handle(),
                0,
            );
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
        }

        let sprite_shader = Shader::load("", "")?;
        let screen_shader = Shader::load("", "")?;

        Ok(Self {
            framebuffer,
            texture,
            sprite_shader,
            screen_shader,
        })
    }
}
