use glow::{HasContext, NativeBuffer, NativeVertexArray};
use image::EncodableLayout;

pub(crate) struct Line {
    vao: NativeVertexArray,
    vbo: NativeBuffer,
}

impl Line {
    const VERTEX_DATA: [f32; 8] = [0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 1.0, 1.0];

    pub fn new(gl: &glow::Context) -> Result<Self, String> {
        let vao = unsafe { gl.create_vertex_array()? };
        let vbo = unsafe { gl.create_buffer()? };

        unsafe {
            gl.bind_vertex_array(Some(vao));
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
            gl.buffer_data_u8_slice(
                glow::ARRAY_BUFFER,
                Self::VERTEX_DATA.as_bytes(),
                glow::STATIC_DRAW,
            );
            gl.bind_vertex_array(None);
        }

        Ok(Self { vao, vbo })
    }

    pub fn bind_vao(&self, gl: &glow::Context) {
        unsafe {
            gl.bind_vertex_array(Some(self.vao));
        }
    }

    pub fn bind_vbo(&self, gl: &glow::Context) {
        unsafe {
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.vbo));
        }
    }

    pub fn draw(&self, gl: &glow::Context) {
        unsafe {
            gl.draw_arrays(glow::TRIANGLE_STRIP, 0, 4);
        }
    }
}
