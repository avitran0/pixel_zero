use glam::{Mat4, Vec2, Vec3, Vec4};
use glow::{HasContext, NativeProgram, NativeShader, NativeUniformLocation};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ShaderError {
    #[error("OpenGL error: {0}")]
    OpenGL(String),
    #[error("Shader compilation error: {0}")]
    Compile(String),
    #[error("Shader linking error: {0}")]
    Linking(String),
}

pub(crate) struct Shader {
    program: NativeProgram,
}

impl Shader {
    pub(crate) fn load(
        gl: &glow::Context,
        vertex: &str,
        fragment: &str,
    ) -> Result<Self, ShaderError> {
        let vertex = Self::compile(gl, vertex, glow::VERTEX_SHADER)?;
        let fragment = Self::compile(gl, fragment, glow::FRAGMENT_SHADER)?;
        let program = Self::link(gl, vertex, fragment)?;

        unsafe {
            gl.delete_shader(vertex);
            gl.delete_shader(fragment);
        }

        Ok(Self { program })
    }

    fn compile(gl: &glow::Context, source: &str, kind: u32) -> Result<NativeShader, ShaderError> {
        let shader = unsafe { gl.create_shader(kind).map_err(ShaderError::OpenGL)? };

        unsafe {
            gl.shader_source(shader, source);
            gl.compile_shader(shader);
        }

        let success = unsafe { gl.get_shader_compile_status(shader) };

        if success {
            Ok(shader)
        } else {
            let log = unsafe { gl.get_shader_info_log(shader) };
            Err(ShaderError::Compile(log))
        }
    }

    fn link(
        gl: &glow::Context,
        vertex: NativeShader,
        fragment: NativeShader,
    ) -> Result<NativeProgram, ShaderError> {
        let program = unsafe { gl.create_program().map_err(ShaderError::Linking)? };

        unsafe {
            gl.attach_shader(program, vertex);
            gl.attach_shader(program, fragment);
            gl.link_program(program);

            gl.detach_shader(program, vertex);
            gl.detach_shader(program, fragment);
        }

        let success = unsafe { gl.get_program_link_status(program) };

        if success {
            Ok(program)
        } else {
            let log = unsafe { gl.get_program_info_log(program) };
            Err(ShaderError::Linking(log))
        }
    }

    pub fn bind(&self, gl: &glow::Context) {
        unsafe {
            gl.use_program(Some(self.program));
        }
    }

    pub fn unbind(gl: &glow::Context) {
        unsafe {
            gl.use_program(None);
        }
    }

    pub fn attributes(&self, gl: &glow::Context, attributes: &[VertexAttribute]) {
        let stride = attributes
            .iter()
            .map(VertexAttribute::size_bytes)
            .reduce(|acc, e| acc + e)
            .unwrap() as i32;

        let mut offset = 0;
        for (index, attribute) in attributes.iter().enumerate() {
            unsafe {
                gl.vertex_attrib_pointer_f32(
                    index as u32,
                    attribute.gl_size(),
                    glow::FLOAT,
                    false,
                    stride,
                    offset,
                );

                gl.enable_vertex_attrib_array(index as u32);
            }
            offset += attribute.size_bytes() as i32;
        }
    }

    fn uniform_location(&self, gl: &glow::Context, name: &str) -> Option<NativeUniformLocation> {
        unsafe { gl.get_uniform_location(self.program, name) }
    }

    pub fn set_uniform(&self, gl: &glow::Context, name: &str, uniform: Uniform) {
        if let Some(location) = self.uniform_location(gl, name) {
            uniform.set(gl, location);
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub(crate) enum Uniform {
    Int(i32),
    Float(f32),
    Vec2(Vec2),
    Vec3(Vec3),
    Vec4(Vec4),
    Mat4(Mat4),
}

impl Uniform {
    fn set(&self, gl: &glow::Context, location: NativeUniformLocation) {
        unsafe {
            match self {
                Self::Int(int) => gl.uniform_1_i32(Some(&location), *int),
                Self::Float(float) => gl.uniform_1_f32(Some(&location), *float),
                Self::Vec2(vec) => gl.uniform_2_f32(Some(&location), vec.x, vec.y),
                Self::Vec3(vec) => gl.uniform_3_f32(Some(&location), vec.x, vec.y, vec.z),
                Self::Vec4(vec) => gl.uniform_4_f32(Some(&location), vec.x, vec.y, vec.z, vec.w),
                Self::Mat4(mat) => {
                    gl.uniform_matrix_4_f32_slice(Some(&location), false, &mat.to_cols_array());
                }
            }
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub(crate) enum VertexAttribute {
    Float,
    Vec2,
    Vec3,
    Vec4,
}

impl VertexAttribute {
    fn gl_size(&self) -> i32 {
        match self {
            Self::Float => 1,
            Self::Vec2 => 2,
            Self::Vec3 => 3,
            Self::Vec4 => 4,
        }
    }

    fn size_bytes(&self) -> usize {
        match self {
            Self::Float => size_of::<f32>(),
            Self::Vec2 => size_of::<Vec2>(),
            Self::Vec3 => size_of::<Vec3>(),
            Self::Vec4 => size_of::<Vec4>(),
        }
    }
}
