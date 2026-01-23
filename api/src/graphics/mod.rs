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

/// Width of the virtual framebuffer (GBA-style resolution)
const FB_WIDTH: u32 = 320;
/// Height of the virtual framebuffer (GBA-style resolution)
const FB_HEIGHT: u32 = 240;

/// Graphics context that provides low-level access to the display and
/// a 2D sprite-based rendering API similar to the Game Boy Advance.
///
/// The rendering target is 320x240 pixels (using OpenGL FBO) and is automatically 
/// letterboxed to fit the physical display while maintaining the correct aspect ratio.
/// All rendering is done using OpenGL ES primitives and textures.
pub struct GraphicsContext {
    drm: Drm,
    gbm: Gbm,
    egl: Egl,

    framebuffer: framebuffer::Handle,
    buffer_object: BufferObject<()>,

    // OpenGL rendering state
    fbo: u32,              // Framebuffer object for 320x240 rendering
    fbo_texture: u32,      // Color attachment for FBO
    sprite_shader: u32,    // Shader for rendering sprites
    screen_shader: u32,    // Shader for rendering FBO to screen
    quad_vao: u32,         // VAO for rendering quads
    quad_vbo: u32,         // VBO for quad vertices
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
        let (fbo, fbo_texture, sprite_shader, screen_shader, quad_vao, quad_vbo) = 
            unsafe { Self::init_2d_resources()? };

        Ok(Self {
            drm,
            gbm,
            egl,
            framebuffer,
            buffer_object,
            fbo,
            fbo_texture,
            sprite_shader,
            screen_shader,
            quad_vao,
            quad_vbo,
        })
    }

    unsafe fn init_2d_resources() -> anyhow::Result<(u32, u32, u32, u32, u32, u32)> {
        // Create framebuffer object for 320x240 rendering
        let mut fbo = 0;
        gl::GenFramebuffers(1, &mut fbo);
        gl::BindFramebuffer(gl::FRAMEBUFFER, fbo);

        // Create texture for FBO color attachment
        let mut fbo_texture = 0;
        gl::GenTextures(1, &mut fbo_texture);
        gl::BindTexture(gl::TEXTURE_2D, fbo_texture);
        gl::TexImage2D(
            gl::TEXTURE_2D,
            0,
            gl::RGBA as i32,
            FB_WIDTH as i32,
            FB_HEIGHT as i32,
            0,
            gl::RGBA,
            gl::UNSIGNED_BYTE,
            std::ptr::null(),
        );
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);

        // Attach texture to FBO
        gl::FramebufferTexture2D(
            gl::FRAMEBUFFER,
            gl::COLOR_ATTACHMENT0,
            gl::TEXTURE_2D,
            fbo_texture,
            0,
        );

        // Check FBO status
        let status = gl::CheckFramebufferStatus(gl::FRAMEBUFFER);
        if status != gl::FRAMEBUFFER_COMPLETE {
            return Err(anyhow::anyhow!("Framebuffer is not complete: {}", status));
        }

        gl::BindFramebuffer(gl::FRAMEBUFFER, 0);

        // Create sprite shader (for textured quads)
        let sprite_shader = Self::create_sprite_shader()?;
        
        // Create screen shader (for rendering FBO to screen)
        let screen_shader = Self::create_screen_shader()?;

        // Create VAO and VBO for rendering quads
        let mut quad_vao = 0;
        let mut quad_vbo = 0;
        gl::GenVertexArrays(1, &mut quad_vao);
        gl::GenBuffers(1, &mut quad_vbo);

        gl::BindVertexArray(quad_vao);
        gl::BindBuffer(gl::ARRAY_BUFFER, quad_vbo);

        // Reserve space for dynamic quad vertices (will be updated per draw call)
        gl::BufferData(
            gl::ARRAY_BUFFER,
            (24 * std::mem::size_of::<f32>()) as isize,
            std::ptr::null(),
            gl::DYNAMIC_DRAW,
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

        Ok((fbo, fbo_texture, sprite_shader, screen_shader, quad_vao, quad_vbo))
    }

    unsafe fn create_sprite_shader() -> anyhow::Result<u32> {
        let vertex_shader = Self::compile_shader(gl::VERTEX_SHADER, SPRITE_VERTEX_SHADER)?;
        let fragment_shader = Self::compile_shader(gl::FRAGMENT_SHADER, SPRITE_FRAGMENT_SHADER)?;
        let program = gl::CreateProgram();
        gl::AttachShader(program, vertex_shader);
        gl::AttachShader(program, fragment_shader);
        gl::LinkProgram(program);

        let mut success = 0;
        gl::GetProgramiv(program, gl::LINK_STATUS, &mut success);
        if success == 0 {
            return Err(anyhow::anyhow!("Failed to link sprite shader program"));
        }

        gl::DeleteShader(vertex_shader);
        gl::DeleteShader(fragment_shader);
        Ok(program)
    }

    unsafe fn create_screen_shader() -> anyhow::Result<u32> {
        let vertex_shader = Self::compile_shader(gl::VERTEX_SHADER, SCREEN_VERTEX_SHADER)?;
        let fragment_shader = Self::compile_shader(gl::FRAGMENT_SHADER, SCREEN_FRAGMENT_SHADER)?;
        let program = gl::CreateProgram();
        gl::AttachShader(program, vertex_shader);
        gl::AttachShader(program, fragment_shader);
        gl::LinkProgram(program);

        let mut success = 0;
        gl::GetProgramiv(program, gl::LINK_STATUS, &mut success);
        if success == 0 {
            return Err(anyhow::anyhow!("Failed to link screen shader program"));
        }

        gl::DeleteShader(vertex_shader);
        gl::DeleteShader(fragment_shader);
        Ok(program)
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
        unsafe {
            gl::ClearColor(0.0, 0.0, 0.0, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT)
        };
    }

    /// Clear the framebuffer with a color
    pub fn clear_framebuffer(&mut self, color: Color) {
        unsafe {
            // Bind FBO and clear it
            gl::BindFramebuffer(gl::FRAMEBUFFER, self.fbo);
            let rgba = color.as_f32_array();
            gl::ClearColor(rgba[0], rgba[1], rgba[2], rgba[3]);
            gl::Clear(gl::COLOR_BUFFER_BIT);
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
        }
    }

    /// Draw a sprite at the given position using OpenGL
    /// 
    /// Note: This creates a new texture for each draw call. For optimal performance
    /// with frequently drawn sprites, consider pre-creating textures and storing them.
    /// This approach is simple and works well for GBA-style games with few sprites per frame.
    pub fn draw_sprite(&mut self, sprite: &Sprite, x: i32, y: i32) {
        unsafe {
            // Bind to FBO for rendering
            gl::BindFramebuffer(gl::FRAMEBUFFER, self.fbo);
            gl::Viewport(0, 0, FB_WIDTH as i32, FB_HEIGHT as i32);

            // Create texture from sprite data
            // Note: For better performance, textures could be cached
            let mut texture = 0;
            gl::GenTextures(1, &mut texture);
            gl::BindTexture(gl::TEXTURE_2D, texture);
            
            // Convert sprite pixels to RGBA u8 array
            let mut rgba_data = Vec::with_capacity((sprite.width() * sprite.height() * 4) as usize);
            for pixel in sprite.pixels() {
                let rgba = pixel.as_u8_array();
                rgba_data.extend_from_slice(&rgba);
            }
            
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGBA as i32,
                sprite.width() as i32,
                sprite.height() as i32,
                0,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                rgba_data.as_ptr() as *const _,
            );
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);

            // Use sprite shader
            gl::UseProgram(self.sprite_shader);
            
            // Bind texture to texture unit 0
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, texture);
            let tex_loc = gl::GetUniformLocation(self.sprite_shader, b"spriteTexture\0".as_ptr() as *const _);
            gl::Uniform1i(tex_loc, 0);

            // Set projection matrix (orthographic for 320x240)
            let projection_loc = gl::GetUniformLocation(self.sprite_shader, b"projection\0".as_ptr() as *const _);
            #[rustfmt::skip]
            let projection: [f32; 16] = [
                2.0 / FB_WIDTH as f32, 0.0, 0.0, 0.0,
                0.0, -2.0 / FB_HEIGHT as f32, 0.0, 0.0,
                0.0, 0.0, -1.0, 0.0,
                -1.0, 1.0, 0.0, 1.0,
            ];
            gl::UniformMatrix4fv(projection_loc, 1, gl::FALSE, projection.as_ptr());

            // Calculate quad vertices for the sprite
            let x0 = x as f32;
            let y0 = y as f32;
            let x1 = x0 + sprite.width() as f32;
            let y1 = y0 + sprite.height() as f32;

            #[rustfmt::skip]
            let vertices: [f32; 24] = [
                // positions  // texcoords
                x0, y0,       0.0, 0.0,
                x0, y1,       0.0, 1.0,
                x1, y1,       1.0, 1.0,

                x0, y0,       0.0, 0.0,
                x1, y1,       1.0, 1.0,
                x1, y0,       1.0, 0.0,
            ];

            // Upload vertices
            gl::BindVertexArray(self.quad_vao);
            gl::BindBuffer(gl::ARRAY_BUFFER, self.quad_vbo);
            gl::BufferSubData(
                gl::ARRAY_BUFFER,
                0,
                (vertices.len() * std::mem::size_of::<f32>()) as isize,
                vertices.as_ptr() as *const _,
            );

            // Draw
            gl::DrawArrays(gl::TRIANGLES, 0, 6);

            // Cleanup
            gl::DeleteTextures(1, &texture);
            gl::BindVertexArray(0);
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
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

    /// Get the dimensions of the framebuffer
    pub fn framebuffer_size(&self) -> (u32, u32) {
        (FB_WIDTH, FB_HEIGHT)
    }

    pub fn present(&mut self) -> anyhow::Result<()> {
        // Render FBO texture to screen with letterboxing
        unsafe {
            // Bind default framebuffer
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);

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

            // Clear with black bars
            gl::ClearColor(0.0, 0.0, 0.0, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);

            // Use screen shader
            gl::UseProgram(self.screen_shader);
            
            // Bind FBO texture to texture unit 0
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, self.fbo_texture);
            let tex_loc = gl::GetUniformLocation(self.screen_shader, b"screenTexture\0".as_ptr() as *const _);
            gl::Uniform1i(tex_loc, 0);

            // Draw fullscreen quad
            gl::BindVertexArray(self.quad_vao);
            gl::BindBuffer(gl::ARRAY_BUFFER, self.quad_vbo);
            
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
            
            gl::BufferSubData(
                gl::ARRAY_BUFFER,
                0,
                (vertices.len() * std::mem::size_of::<f32>()) as isize,
                vertices.as_ptr() as *const _,
            );

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

// Vertex shader for rendering sprites
const SPRITE_VERTEX_SHADER: &str = r#"#version 320 es
precision mediump float;

layout (location = 0) in vec2 aPos;
layout (location = 1) in vec2 aTexCoord;

uniform mat4 projection;

out vec2 TexCoord;

void main() {
    gl_Position = projection * vec4(aPos, 0.0, 1.0);
    TexCoord = aTexCoord;
}
"#;

// Fragment shader for rendering sprites
const SPRITE_FRAGMENT_SHADER: &str = r#"#version 320 es
precision mediump float;

in vec2 TexCoord;
out vec4 FragColor;

uniform sampler2D spriteTexture;

void main() {
    FragColor = texture(spriteTexture, TexCoord);
}
"#;

// Vertex shader for rendering FBO to screen
const SCREEN_VERTEX_SHADER: &str = r#"#version 320 es
precision mediump float;

layout (location = 0) in vec2 aPos;
layout (location = 1) in vec2 aTexCoord;

out vec2 TexCoord;

void main() {
    gl_Position = vec4(aPos, 0.0, 1.0);
    TexCoord = aTexCoord;
}
"#;

// Fragment shader for rendering FBO to screen
const SCREEN_FRAGMENT_SHADER: &str = r#"#version 320 es
precision mediump float;

in vec2 TexCoord;
out vec4 FragColor;

uniform sampler2D screenTexture;

void main() {
    FragColor = texture(screenTexture, TexCoord);
}
"#;
