use glam::{Mat4, UVec2, uvec2};

use crate::{
    HEIGHT, WIDTH,
    graphics::{
        color::Color,
        shader::{Shader, ShaderError},
        texture::Texture,
    },
};

pub(crate) struct Framebuffer {
    framebuffer: u32,
    texture: Texture,
    sprite_shader: Shader,
    screen_shader: Shader,
    screen_size: UVec2,
}

impl Framebuffer {
    pub fn load(screen_size: UVec2) -> Result<Self, ShaderError> {
        let mut framebuffer = 0;
        unsafe {
            gl::GenFramebuffers(1, &raw mut framebuffer);
            gl::BindFramebuffer(gl::FRAMEBUFFER, framebuffer);
        }

        let texture = Texture::empty(uvec2(WIDTH, HEIGHT));

        unsafe {
            gl::FramebufferTexture2D(
                gl::FRAMEBUFFER,
                gl::COLOR_ATTACHMENT0,
                gl::TEXTURE_2D,
                texture.handle(),
                0,
            );

            let status = gl::CheckFramebufferStatus(gl::FRAMEBUFFER);
            if status != gl::FRAMEBUFFER_COMPLETE {
                return Err(ShaderError::Linking(format!(
                    "Framebuffer Incomplete: 0x{status:X}"
                )));
            }

            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
        }

        let mut sprite_shader = Shader::load(
            include_str!("shaders/sprite.vert"),
            include_str!("shaders/sprite.frag"),
        )?;
        let mut screen_shader = Shader::load(
            include_str!("shaders/screen.vert"),
            include_str!("shaders/screen.frag"),
        )?;

        sprite_shader.bind();
        sprite_shader.set_attribute("a_position", 0, 2, 4, 0, gl::FLOAT);
        sprite_shader.set_attribute("a_texcoord", 1, 2, 4, 2, gl::FLOAT);
        let projection = Mat4::orthographic_rh(0.0, WIDTH as f32, HEIGHT as f32, 0.0, -1.0, 1.0);
        sprite_shader.set_mat4("u_projection", &projection);

        screen_shader.bind();
        screen_shader.set_attribute("a_position", 0);
        screen_shader.set_attribute("a_texcoord", 1);
        screen_shader.set_vec2("u_screen_size", screen_size.as_vec2());
        Shader::unbind();

        Ok(Self {
            framebuffer,
            texture,
            sprite_shader,
            screen_shader,
            screen_size,
        })
    }

    fn bind(&self) {
        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, self.framebuffer);
            gl::Viewport(0, 0, WIDTH.cast_signed(), HEIGHT.cast_signed());
        }
    }

    fn unbind(&self) {
        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
            gl::Viewport(
                0,
                0,
                self.screen_size.x.cast_signed(),
                self.screen_size.y.cast_signed(),
            );
        }
    }

    pub(crate) fn clear(&self, color: Color) {
        let color = color.f32();
        unsafe {
            gl::ClearColor(color.r(), color.g(), color.b(), color.a());
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }
    }
}

impl Drop for Framebuffer {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteFramebuffers(1, &raw const self.framebuffer);
        }
    }
}
