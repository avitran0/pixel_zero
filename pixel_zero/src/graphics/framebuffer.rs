use glam::{Mat4, UVec2, ivec2, uvec2};
use glow::{HasContext, NativeFramebuffer};
use thiserror::Error;

use crate::{
    HEIGHT, WIDTH,
    graphics::{
        color::Color,
        frame::{DrawCommand, Frame},
        quad::Quad,
        shader::{Shader, ShaderError, Uniform, VertexAttribute},
        texture::{Texture, TextureError},
    },
};

#[derive(Debug, Error)]
pub enum FramebufferError {
    #[error("OpenGL error: {0}")]
    OpenGL(String),
    #[error("{0}")]
    Shader(#[from] ShaderError),
    #[error("{0}")]
    Texture(#[from] TextureError),
}

pub(crate) struct Framebuffer {
    framebuffer: NativeFramebuffer,
    texture: Texture,
    sprite_shader: Shader,
    screen_shader: Shader,
    screen_size: UVec2,
    quad: Quad,
}

impl Framebuffer {
    pub fn load(gl: &glow::Context, screen_size: UVec2) -> Result<Self, FramebufferError> {
        let framebuffer = unsafe { gl.create_framebuffer().map_err(ShaderError::OpenGL)? };
        unsafe {
            gl.bind_framebuffer(glow::FRAMEBUFFER, Some(framebuffer));
        }

        let texture = Texture::load_empty(gl, uvec2(WIDTH, HEIGHT))?;

        unsafe {
            gl.framebuffer_texture_2d(
                glow::FRAMEBUFFER,
                glow::COLOR_ATTACHMENT0,
                glow::TEXTURE_2D,
                Some(texture.handle()),
                0,
            );

            let status = gl.check_named_framebuffer_status(Some(framebuffer), glow::FRAMEBUFFER);
            if status != glow::FRAMEBUFFER_COMPLETE {
                return Err(FramebufferError::Shader(ShaderError::Linking(format!(
                    "Framebuffer Incomplete: 0x{status:X}"
                ))));
            }

            gl.bind_framebuffer(glow::FRAMEBUFFER, None);
        }

        let sprite_shader = Shader::load(
            gl,
            include_str!("shaders/sprite.vert"),
            include_str!("shaders/sprite.frag"),
        )?;
        let screen_shader = Shader::load(
            gl,
            include_str!("shaders/screen.vert"),
            include_str!("shaders/screen.frag"),
        )?;

        let quad = Quad::new(gl).map_err(FramebufferError::OpenGL)?;
        quad.bind_vao(gl);
        quad.bind_vbo(gl);

        sprite_shader.bind(gl);
        sprite_shader.attributes(gl, &[VertexAttribute::Vec2, VertexAttribute::Vec2]);
        let projection = Mat4::orthographic_rh(0.0, WIDTH as f32, HEIGHT as f32, 0.0, -1.0, 1.0);
        sprite_shader.set_uniform(gl, "u_projection", Uniform::Mat4(projection));
        sprite_shader.set_uniform(gl, "u_color", Uniform::Vec3(Color::WHITE.vec3()));
        sprite_shader.set_uniform(gl, "u_texture", Uniform::Int(0));

        screen_shader.bind(gl);
        screen_shader.attributes(gl, &[VertexAttribute::Vec2, VertexAttribute::Vec2]);
        screen_shader.set_uniform(gl, "u_screen_size", Uniform::Vec2(screen_size.as_vec2()));
        screen_shader.set_uniform(gl, "u_texture", Uniform::Int(0));

        Shader::unbind(gl);
        Quad::unbind_vao(gl);
        Quad::unbind_vbo(gl);

        unsafe {
            // gl::PolygonMode(gl::FRONT_AND_BACK, gl::LINE);
            gl.active_texture(glow::TEXTURE0);
            gl.enable(glow::BLEND);
            gl.blend_func(glow::SRC_ALPHA, glow::ONE_MINUS_SRC_ALPHA);
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

    fn bind(&self, gl: &glow::Context) {
        unsafe {
            gl.bind_framebuffer(glow::FRAMEBUFFER, Some(self.framebuffer));
            gl.viewport(0, 0, WIDTH.cast_signed(), HEIGHT.cast_signed());
        }
    }

    fn unbind(&self, gl: &glow::Context) {
        unsafe {
            gl.bind_framebuffer(glow::FRAMEBUFFER, None);
            gl.viewport(
                0,
                0,
                self.screen_size.x.cast_signed(),
                self.screen_size.y.cast_signed(),
            );
        }
    }

    pub(crate) fn present_frame(&self, gl: &glow::Context, frame: &Frame) {
        self.bind(gl);
        self.sprite_shader.bind(gl);
        self.quad.bind_vao(gl);

        let color = frame.clear_color().f32();
        unsafe {
            gl.clear_color(color.r(), color.g(), color.b(), color.a());
            gl.clear(glow::COLOR_BUFFER_BIT);
        }

        for command in frame.commands() {
            match command {
                DrawCommand::Sprite { sprite, position } => {
                    self.sprite_shader.set_uniform(
                        gl,
                        "u_position",
                        Uniform::Vec2(position.as_vec2()),
                    );
                    self.sprite_shader.set_uniform(
                        gl,
                        "u_size",
                        Uniform::Vec2(sprite.texture().size().as_vec2()),
                    );
                    self.sprite_shader.set_uniform(
                        gl,
                        "u_texcoords",
                        Uniform::Vec4(sprite.region().vec4()),
                    );
                    sprite.texture().bind(gl);
                    self.quad.draw(gl);
                }
                DrawCommand::Text {
                    font,
                    text,
                    position,
                } => {
                    self.sprite_shader.set_uniform(
                        gl,
                        "u_size",
                        Uniform::Vec2(font.glyph_size().as_vec2()),
                    );
                    font.texture().bind(gl);

                    let mut advance = 0;
                    for char in text.chars() {
                        let glyph = font.glyph(char).unwrap_or(font.default_glyph());

                        let char_position = *position + ivec2(advance, 0);
                        self.sprite_shader.set_uniform(
                            gl,
                            "u_position",
                            Uniform::Vec2(char_position.as_vec2()),
                        );

                        self.sprite_shader.set_uniform(
                            gl,
                            "u_texcoords",
                            Uniform::Vec4(glyph.region().vec4()),
                        );

                        self.quad.draw(gl);

                        advance += glyph.advance().cast_signed();
                    }
                }
            }
        }

        self.unbind(gl);

        self.texture.bind(gl);
        self.screen_shader.bind(gl);
        self.quad.bind_vao(gl);

        self.quad.draw(gl);

        Texture::unbind(gl);
        Quad::unbind_vao(gl);
        Shader::unbind(gl);
    }
}
