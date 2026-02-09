pub(crate) struct Quad {
    vbo: u32,
    vao: u32,
}

impl Quad {
    const VERTEX_DATA: [f32; 24] = [
        0.0, 1.0, 0.0, 1.0, 1.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 1.0, 1.0, 1.0,
        1.0, 1.0, 1.0, 0.0, 1.0, 0.0,
    ];

    pub fn new() -> Self {
        let mut vao = 0;
        let mut vbo = 0;

        unsafe {
            gl::GenVertexArrays(1, &raw mut vao);
            gl::GenBuffers(1, &raw mut vbo);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                size_of_val(&Self::VERTEX_DATA).cast_signed(),
                Self::VERTEX_DATA.as_ref().as_ptr().cast(),
                gl::STATIC_DRAW,
            );
        }

        Self { vbo, vao }
    }

    pub fn bind_vao(&self) {
        unsafe {
            gl::BindVertexArray(self.vao);
        }
    }

    pub fn bind_vbo(&self) {
        unsafe {
            gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);
        }
    }

    pub fn unbind_vao() {
        unsafe {
            gl::BindVertexArray(0);
        }
    }

    pub fn unbind_vbo() {
        unsafe {
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        }
    }
}

impl Drop for Quad {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteBuffers(1, &raw const self.vbo);
            gl::DeleteVertexArrays(1, &raw const self.vao);
        }
    }
}
