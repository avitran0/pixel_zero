use glam::{IVec2, Mat4, UVec2, Vec2, ivec2, uvec2};
use glow::{HasContext, NativeFramebuffer};
use thiserror::Error;

use crate::{
    HEIGHT, WIDTH,
    graphics::{
        Font, Sprite,
        color::Color,
        frame::{DrawCommand, Frame},
        line::Line,
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
    shape_shader: Shader,
    screen_shader: Shader,
    screen_size: UVec2,
    quad: Quad,
    line: Line,
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

            let status = gl.check_framebuffer_status(glow::FRAMEBUFFER);
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
        let shape_shader = Shader::load(
            gl,
            include_str!("shaders/shape.vert"),
            include_str!("shaders/shape.frag"),
        )?;
        let screen_shader = Shader::load(
            gl,
            include_str!("shaders/screen.vert"),
            include_str!("shaders/screen.frag"),
        )?;

        let quad = Quad::new(gl).map_err(FramebufferError::OpenGL)?;
        let line = Line::new(gl).map_err(FramebufferError::OpenGL)?;

        let projection = Mat4::orthographic_rh(0.0, WIDTH as f32, HEIGHT as f32, 0.0, -1.0, 1.0);

        // quad has position + uv
        quad.bind_vao(gl);
        quad.bind_vbo(gl);
        sprite_shader.attributes(gl, &[VertexAttribute::Vec2, VertexAttribute::Vec2]);

        // line only has position
        line.bind_vao(gl);
        line.bind_vbo(gl);
        shape_shader.attributes(gl, &[VertexAttribute::Vec2]);

        sprite_shader.bind(gl);
        sprite_shader.set_uniform(gl, "u_projection", Uniform::Mat4(projection));
        sprite_shader.set_uniform(gl, "u_color", Uniform::Vec3(Color::WHITE.vec3()));
        sprite_shader.set_uniform(gl, "u_texture", Uniform::Int(0));

        shape_shader.bind(gl);
        shape_shader.set_uniform(gl, "u_projection", Uniform::Mat4(projection));

        screen_shader.bind(gl);
        screen_shader.set_uniform(gl, "u_screen_size", Uniform::Vec2(screen_size.as_vec2()));
        screen_shader.set_uniform(gl, "u_texture", Uniform::Int(0));

        Shader::unbind(gl);
        Quad::unbind_vao(gl);
        Quad::unbind_vbo(gl);

        unsafe {
            gl.active_texture(glow::TEXTURE0);
            gl.enable(glow::BLEND);
            gl.blend_func(glow::SRC_ALPHA, glow::ONE_MINUS_SRC_ALPHA);
        }

        Ok(Self {
            framebuffer,
            texture,
            sprite_shader,
            shape_shader,
            screen_shader,
            screen_size,
            quad,
            line,
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

        let color = frame.clear_color().f32();
        unsafe {
            gl.clear_color(color.r(), color.g(), color.b(), color.a());
            gl.clear(glow::COLOR_BUFFER_BIT);
        }

        for command in frame.commands() {
            match command {
                DrawCommand::Sprite { sprite, position } => {
                    self.draw_sprite(gl, sprite, *position);
                }
                DrawCommand::Text {
                    font,
                    text,
                    position,
                } => {
                    self.draw_text(gl, font, text, *position);
                }
                DrawCommand::Line {
                    start,
                    end,
                    width,
                    color,
                } => {
                    self.draw_line(gl, *start, *end, *width, *color);
                }
                DrawCommand::Rect {
                    position,
                    size,
                    color,
                    filled,
                } => {
                    if *filled {
                        self.draw_rect_filled(gl, *position, *size, *color);
                    } else {
                        self.draw_rect(gl, *position, *size, *color);
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

    fn draw_sprite(&self, gl: &glow::Context, sprite: &Sprite, position: IVec2) {
        self.sprite_shader.bind(gl);
        self.quad.bind_vao(gl);

        self.sprite_shader
            .set_uniform(gl, "u_position", Uniform::Vec2(position.as_vec2()));
        self.sprite_shader.set_uniform(
            gl,
            "u_size",
            Uniform::Vec2(sprite.texture().size().as_vec2()),
        );
        self.sprite_shader
            .set_uniform(gl, "u_texcoords", Uniform::Vec4(sprite.region().vec4()));
        sprite.texture().bind(gl);
        self.quad.draw(gl);
    }

    fn draw_text(&self, gl: &glow::Context, font: &Font, text: &str, position: IVec2) {
        self.sprite_shader.bind(gl);
        self.quad.bind_vao(gl);

        self.sprite_shader
            .set_uniform(gl, "u_size", Uniform::Vec2(font.glyph_size().as_vec2()));
        font.texture().bind(gl);

        let mut advance = 0;
        for char in text.chars() {
            let glyph = font.glyph(char).unwrap_or(font.default_glyph());

            let char_position = position + ivec2(advance, 0);
            self.sprite_shader.set_uniform(
                gl,
                "u_position",
                Uniform::Vec2(char_position.as_vec2()),
            );

            self.sprite_shader
                .set_uniform(gl, "u_texcoords", Uniform::Vec4(glyph.region().vec4()));

            self.quad.draw(gl);

            advance += glyph.advance().cast_signed();
        }
    }

    fn draw_line(&self, gl: &glow::Context, start: IVec2, end: IVec2, _width: u32, color: Color) {
        self.shape_shader.bind(gl);
        self.line.bind_vao(gl);

        self.shape_shader
            .set_uniform(gl, "u_color", Uniform::Vec4(color.vec4()));

        let start_f = start.as_vec2();
        let end_f = end.as_vec2();
        let size = end_f - start_f;

        self.shape_shader
            .set_uniform(gl, "u_position", Uniform::Vec2(start_f));
        self.shape_shader
            .set_uniform(gl, "u_size", Uniform::Vec2(size));

        self.line.draw(gl);
    }

    fn draw_rect(&self, gl: &glow::Context, position: IVec2, size: UVec2, color: Color) {
        self.shape_shader.bind(gl);
        self.line.bind_vao(gl);

        self.shape_shader
            .set_uniform(gl, "u_color", Uniform::Vec4(color.vec4()));

        let x = position.x as f32;
        let y = position.y as f32;
        let w = size.x as f32;
        let h = size.y as f32;

        // top: (x, y) -> (x + w, y)
        self.shape_shader
            .set_uniform(gl, "u_position", Uniform::Vec2(Vec2::new(x, y)));
        self.shape_shader
            .set_uniform(gl, "u_size", Uniform::Vec2(Vec2::new(w, 0.0)));
        self.line.draw(gl);

        // bottom: (x, y + h) -> (x + w, y + h)
        self.shape_shader
            .set_uniform(gl, "u_position", Uniform::Vec2(Vec2::new(x, y + h)));
        self.shape_shader
            .set_uniform(gl, "u_size", Uniform::Vec2(Vec2::new(w, 0.0)));
        self.line.draw(gl);

        // left: (x, y) -> (x, y + h)
        self.shape_shader
            .set_uniform(gl, "u_position", Uniform::Vec2(Vec2::new(x, y)));
        self.shape_shader
            .set_uniform(gl, "u_size", Uniform::Vec2(Vec2::new(0.0, h)));
        self.line.draw(gl);

        // right: (x + w, y) -> (x + w, y + h)
        self.shape_shader
            .set_uniform(gl, "u_position", Uniform::Vec2(Vec2::new(x + w, y)));
        self.shape_shader
            .set_uniform(gl, "u_size", Uniform::Vec2(Vec2::new(0.0, h)));
        self.line.draw(gl);
    }

    fn draw_rect_filled(&self, gl: &glow::Context, position: IVec2, size: UVec2, color: Color) {
        self.shape_shader.bind(gl);
        self.quad.bind_vao(gl);

        self.shape_shader
            .set_uniform(gl, "u_color", Uniform::Vec4(color.vec4()));

        self.shape_shader
            .set_uniform(gl, "u_position", Uniform::Vec2(position.as_vec2()));
        self.shape_shader
            .set_uniform(gl, "u_size", Uniform::Vec2(size.as_vec2()));

        self.quad.draw(gl);
    }
}
