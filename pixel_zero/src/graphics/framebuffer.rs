use glam::{Mat4, UVec2, uvec2};

use crate::{
    HEIGHT, WIDTH,
    graphics::{
        color::Color,
        quad::Quad,
        shader::{Shader, ShaderError, Uniform, VertexAttribute},
        sprite::Sprite,
        texture::Texture,
    },
};

pub(crate) struct Framebuffer {
    framebuffer: u32,
    texture: Texture,
    sprite_shader: Shader,
    screen_shader: Shader,
    screen_size: UVec2,
    quad: Quad,
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

        let sprite_shader = Shader::load(
            include_str!("shaders/sprite.vert"),
            include_str!("shaders/sprite.frag"),
        )?;
        let screen_shader = Shader::load(
            include_str!("shaders/screen.vert"),
            include_str!("shaders/screen.frag"),
        )?;

        let quad = Quad::new();
        quad.bind_vao();
        quad.bind_vbo();

        sprite_shader.bind();
        sprite_shader.attributes(&[VertexAttribute::Vec2, VertexAttribute::Vec2]);
        let projection = Mat4::orthographic_rh(0.0, WIDTH as f32, HEIGHT as f32, 0.0, -1.0, 1.0);
        sprite_shader.set_uniform("u_projection", &Uniform::Mat4(projection));
        sprite_shader.set_uniform("u_color", &Uniform::Vec3(Color::WHITE.vec3()));
        sprite_shader.set_uniform("u_texture", &Uniform::Int(0));

        screen_shader.bind();
        screen_shader.attributes(&[VertexAttribute::Vec2, VertexAttribute::Vec2]);
        screen_shader.set_uniform("u_screen_size", &Uniform::Vec2(screen_size.as_vec2()));
        screen_shader.set_uniform("u_texture", &Uniform::Int(0));

        Shader::unbind();
        Quad::unbind_vao();
        Quad::unbind_vbo();

        unsafe {
            // gl::PolygonMode(gl::FRONT_AND_BACK, gl::LINE);
            gl::ActiveTexture(gl::TEXTURE0);
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
        }

        Ok(Self {
            framebuffer,
            texture,
            sprite_shader,
            screen_shader,
            screen_size,
            quad,
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
        self.bind();
        let color = color.f32();
        unsafe {
            gl::ClearColor(color.r(), color.g(), color.b(), color.a());
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }
        self.unbind();
    }

    pub(crate) fn draw_sprite(&self, sprite: &Sprite, position: UVec2) {
        self.bind();
        self.sprite_shader.bind();
        self.sprite_shader
            .set_uniform("u_position", &Uniform::Vec2(position.as_vec2()));
        self.sprite_shader
            .set_uniform("u_size", &Uniform::Vec2(sprite.texture.size().as_vec2()));
        sprite.texture.bind();
        self.quad.bind_vao();

        unsafe {
            gl::DrawArrays(gl::TRIANGLES, 0, 6);
        }

        Texture::unbind();
        Quad::unbind_vao();
        Shader::unbind();
        self.unbind();
    }

    pub(crate) fn present(&self) {
        self.unbind();

        self.texture.bind();
        self.screen_shader.bind();
        self.quad.bind_vao();

        unsafe {
            gl::DrawArrays(gl::TRIANGLES, 0, 6);
        }

        Texture::unbind();
        Quad::unbind_vao();
        Shader::unbind();
    }
}

impl Drop for Framebuffer {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteFramebuffers(1, &raw const self.framebuffer);
        }
    }
}
