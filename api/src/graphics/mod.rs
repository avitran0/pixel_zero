use std::sync::atomic::{AtomicBool, Ordering};

use ::drm::control::{self, Device as _, PageFlipFlags, framebuffer};
use ::gbm::BufferObject;

use crate::graphics::{drm::Drm, egl::Egl, gbm::Gbm, color::Color, sprite::Sprite, font::BitmapFont};

pub mod color;
pub mod sprite;
pub mod font;
mod drm;
mod egl;
mod gbm;

const FB_WIDTH: u32 = 320;
const FB_HEIGHT: u32 = 240;

pub struct GraphicsContext {
    drm: Drm,
    gbm: Gbm,
    egl: Egl,

    framebuffer: framebuffer::Handle,
    buffer_object: BufferObject<()>,

    // 2D rendering state
    virtual_framebuffer: Vec<Color>,
    texture: u32,
    shader_program: u32,
    vao: u32,
    vbo: u32,
}

static LOADED: AtomicBool = AtomicBool::new(false);
impl GraphicsContext {
    pub fn load() -> anyhow::Result<Self> {
        if LOADED.swap(true, Ordering::Relaxed) {
            return Err(anyhow::anyhow!("GraphicsContext already loaded"));
        }

        let drm = Drm::load()?;
        let mut gbm = Gbm::load(&drm)?;
        let egl = Egl::load(&mut gbm)?;

        let buffer_object = unsafe { gbm.surface().lock_front_buffer() }?;
        let bpp = buffer_object.bpp();
        let framebuffer = drm.gpu().add_framebuffer(&buffer_object, bpp, bpp)?;
        drm.gpu().set_crtc(
            drm.crtc().handle(),
            Some(framebuffer),
            (0, 0),
            &[drm.connector().handle()],
            Some(*drm.mode()),
        )?;

        // Initialize 2D rendering resources
        let virtual_framebuffer = vec![Color::BLACK; (FB_WIDTH * FB_HEIGHT) as usize];
        let (texture, shader_program, vao, vbo) = unsafe { Self::init_2d_resources()? };

        Ok(Self {
            drm,
            gbm,
            egl,
            framebuffer,
            buffer_object,
            virtual_framebuffer,
            texture,
            shader_program,
            vao,
            vbo,
        })
    }

    unsafe fn init_2d_resources() -> anyhow::Result<(u32, u32, u32, u32)> {
        // Create texture for the virtual framebuffer
        let mut texture = 0;
        gl::GenTextures(1, &mut texture);
        gl::BindTexture(gl::TEXTURE_2D, texture);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);

        // Create shader program
        let vertex_shader = Self::compile_shader(gl::VERTEX_SHADER, VERTEX_SHADER_SRC)?;
        let fragment_shader = Self::compile_shader(gl::FRAGMENT_SHADER, FRAGMENT_SHADER_SRC)?;
        let shader_program = gl::CreateProgram();
        gl::AttachShader(shader_program, vertex_shader);
        gl::AttachShader(shader_program, fragment_shader);
        gl::LinkProgram(shader_program);

        // Check link status
        let mut success = 0;
        gl::GetProgramiv(shader_program, gl::LINK_STATUS, &mut success);
        if success == 0 {
            return Err(anyhow::anyhow!("Failed to link shader program"));
        }

        gl::DeleteShader(vertex_shader);
        gl::DeleteShader(fragment_shader);

        // Create VAO and VBO for a fullscreen quad
        let mut vao = 0;
        let mut vbo = 0;
        gl::GenVertexArrays(1, &mut vao);
        gl::GenBuffers(1, &mut vbo);

        gl::BindVertexArray(vao);
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);

        // Fullscreen quad vertices (position + texcoord)
        #[rustfmt::skip]
        let vertices: [f32; 24] = [
            // positions   // texcoords
            -1.0,  1.0,    0.0, 0.0,
            -1.0, -1.0,    0.0, 1.0,
             1.0, -1.0,    1.0, 1.0,

            -1.0,  1.0,    0.0, 0.0,
             1.0, -1.0,    1.0, 1.0,
             1.0,  1.0,    1.0, 0.0,
        ];

        gl::BufferData(
            gl::ARRAY_BUFFER,
            (vertices.len() * std::mem::size_of::<f32>()) as isize,
            vertices.as_ptr() as *const _,
            gl::STATIC_DRAW,
        );

        // Position attribute
        gl::VertexAttribPointer(0, 2, gl::FLOAT, gl::FALSE, 4 * std::mem::size_of::<f32>() as i32, std::ptr::null());
        gl::EnableVertexAttribArray(0);

        // Texcoord attribute
        gl::VertexAttribPointer(1, 2, gl::FLOAT, gl::FALSE, 4 * std::mem::size_of::<f32>() as i32, (2 * std::mem::size_of::<f32>()) as *const _);
        gl::EnableVertexAttribArray(1);

        gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        gl::BindVertexArray(0);

        // Enable blending for transparency
        gl::Enable(gl::BLEND);
        gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);

        Ok((texture, shader_program, vao, vbo))
    }

    unsafe fn compile_shader(shader_type: u32, source: &str) -> anyhow::Result<u32> {
        let shader = gl::CreateShader(shader_type);
        let c_str = std::ffi::CString::new(source)?;
        gl::ShaderSource(shader, 1, &c_str.as_ptr(), std::ptr::null());
        gl::CompileShader(shader);

        let mut success = 0;
        gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut success);
        if success == 0 {
            let mut len = 0;
            gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut len);
            let mut buffer = vec![0u8; len as usize];
            gl::GetShaderInfoLog(shader, len, &mut len, buffer.as_mut_ptr() as *mut _);
            let error = String::from_utf8_lossy(&buffer);
            return Err(anyhow::anyhow!("Shader compilation failed: {}", error));
        }

        Ok(shader)
    }

    pub fn clear(&self) {
        // Clear the virtual framebuffer
        for pixel in self.virtual_framebuffer.iter() {
            // We can't modify through immutable reference, need mutable clear
        }
        unsafe {
            gl::ClearColor(0.0, 0.0, 0.0, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT)
        };
    }

    /// Clear the virtual framebuffer with a color
    pub fn clear_framebuffer(&mut self, color: Color) {
        self.virtual_framebuffer.fill(color);
    }

    /// Draw a sprite to the virtual framebuffer at the given position
    pub fn draw_sprite(&mut self, sprite: &Sprite, x: i32, y: i32) {
        for sy in 0..sprite.height() {
            for sx in 0..sprite.width() {
                let dest_x = x + sx as i32;
                let dest_y = y + sy as i32;

                // Clip to framebuffer bounds
                if dest_x < 0 || dest_x >= FB_WIDTH as i32 || dest_y < 0 || dest_y >= FB_HEIGHT as i32 {
                    continue;
                }

                if let Some(pixel) = sprite.get_pixel(sx, sy) {
                    // Skip fully transparent pixels
                    if pixel.a() == 0 {
                        continue;
                    }

                    let index = (dest_y as u32 * FB_WIDTH + dest_x as u32) as usize;
                    if index < self.virtual_framebuffer.len() {
                        // Simple alpha blending
                        if pixel.a() == 255 {
                            self.virtual_framebuffer[index] = pixel;
                        } else {
                            let dst = self.virtual_framebuffer[index];
                            let alpha = pixel.a() as f32 / 255.0;
                            let inv_alpha = 1.0 - alpha;
                            self.virtual_framebuffer[index] = Color::rgba(
                                ((pixel.r() as f32 * alpha + dst.r() as f32 * inv_alpha) as u8),
                                ((pixel.g() as f32 * alpha + dst.g() as f32 * inv_alpha) as u8),
                                ((pixel.b() as f32 * alpha + dst.b() as f32 * inv_alpha) as u8),
                                255,
                            );
                        }
                    }
                }
            }
        }
    }

    /// Draw text using a bitmap font
    pub fn draw_text(&mut self, font: &BitmapFont, text: &str, x: i32, y: i32, color: Color) {
        let mut cursor_x = x;
        for ch in text.chars() {
            if let Some(mut glyph) = font.get_glyph(ch) {
                // Colorize the glyph
                for gy in 0..glyph.height() {
                    for gx in 0..glyph.width() {
                        if let Some(pixel) = glyph.get_pixel(gx, gy) {
                            // Only draw non-transparent pixels with the specified color
                            if pixel.a() > 0 {
                                glyph.set_pixel(gx, gy, Color::rgba(color.r(), color.g(), color.b(), pixel.a()));
                            }
                        }
                    }
                }
                self.draw_sprite(&glyph, cursor_x, y);
                cursor_x += font.glyph_width() as i32;
            }
        }
    }

    /// Get the dimensions of the virtual framebuffer
    pub fn framebuffer_size(&self) -> (u32, u32) {
        (FB_WIDTH, FB_HEIGHT)
    }

    pub fn present(&mut self) -> anyhow::Result<()> {
        // Upload the virtual framebuffer to the texture
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, self.texture);
            
            // Convert Color buffer to RGBA8
            let mut rgba_data = Vec::with_capacity((FB_WIDTH * FB_HEIGHT * 4) as usize);
            for color in &self.virtual_framebuffer {
                let rgba = color.as_u8_array();
                rgba_data.extend_from_slice(&rgba);
            }
            
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGBA as i32,
                FB_WIDTH as i32,
                FB_HEIGHT as i32,
                0,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                rgba_data.as_ptr() as *const _,
            );

            // Calculate letterbox viewport
            let screen_width = self.gbm.size().x;
            let screen_height = self.gbm.size().y;
            let aspect_ratio = FB_WIDTH as f32 / FB_HEIGHT as f32;
            let screen_aspect = screen_width as f32 / screen_height as f32;

            let (vp_width, vp_height, vp_x, vp_y) = if screen_aspect > aspect_ratio {
                // Screen is wider - letterbox on sides
                let vp_width = (screen_height as f32 * aspect_ratio) as u32;
                let vp_x = (screen_width - vp_width) / 2;
                (vp_width, screen_height, vp_x, 0)
            } else {
                // Screen is taller - letterbox on top/bottom
                let vp_height = (screen_width as f32 / aspect_ratio) as u32;
                let vp_y = (screen_height - vp_height) / 2;
                (screen_width, vp_height, 0, vp_y)
            };

            gl::Viewport(vp_x as i32, vp_y as i32, vp_width as i32, vp_height as i32);

            // Render the texture
            gl::UseProgram(self.shader_program);
            gl::BindVertexArray(self.vao);
            gl::DrawArrays(gl::TRIANGLES, 0, 6);
            gl::BindVertexArray(0);
        }

        self.egl
            .instance()
            .swap_buffers(self.egl.display(), self.egl.surface())?;

        let buffer_object = unsafe { self.gbm.surface().lock_front_buffer() }?;
        let bpp = buffer_object.bpp();
        let framebuffer = self.drm.gpu().add_framebuffer(&buffer_object, bpp, bpp)?;

        self.drm.gpu().page_flip(
            self.drm.crtc().handle(),
            framebuffer,
            PageFlipFlags::EVENT,
            None,
        )?;
        let events = self.drm.gpu().receive_events()?;
        for event in events {
            if let control::Event::PageFlip(event) = event {
                // todo
            }
        }

        self.drm.gpu().destroy_framebuffer(self.framebuffer)?;

        self.buffer_object = buffer_object;
        self.framebuffer = framebuffer;

        Ok(())
    }
}

// Vertex shader for rendering the virtual framebuffer
const VERTEX_SHADER_SRC: &str = r#"#version 320 es
precision mediump float;

layout (location = 0) in vec2 aPos;
layout (location = 1) in vec2 aTexCoord;

out vec2 TexCoord;

void main() {
    gl_Position = vec4(aPos, 0.0, 1.0);
    TexCoord = aTexCoord;
}
"#;

// Fragment shader for rendering the virtual framebuffer
const FRAGMENT_SHADER_SRC: &str = r#"#version 320 es
precision mediump float;

in vec2 TexCoord;
out vec4 FragColor;

uniform sampler2D screenTexture;

void main() {
    FragColor = texture(screenTexture, TexCoord);
}
"#;
