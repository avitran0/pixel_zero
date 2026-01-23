use crate::graphics::{shader::Shader, texture::Texture};

pub(crate) struct Framebuffer {
    fbo: u32,
    fbo_texture: Texture,
    sprite_shader: Shader,
    screen_shader: Shader,
}

impl Framebuffer {
    pub fn load() -> anyhow::Result<Self> {
        let mut fbo = 0;
        unsafe {
            gl::GenFramebuffers(1, &mut fbo);
            gl::BindFramebuffer(gl::FRAMEBUFFER, fbo);
        }

        Err(anyhow::anyhow!(""))
    }
}
