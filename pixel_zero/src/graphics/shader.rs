use std::{collections::HashMap, ffi::CString};

use glam::{Mat4, Vec2, Vec3, Vec4};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ShaderError {
    #[error("Shader compilation error: {0}")]
    Compile(String),
    #[error("Shader linking error: {0}")]
    Linking(String),
}

pub struct Shader {
    program: u32,
}

impl Shader {
    pub fn load(vertex: &str, fragment: &str) -> Result<Self, ShaderError> {
        let vertex = Self::compile(vertex, gl::VERTEX_SHADER)?;
        let fragment = Self::compile(fragment, gl::FRAGMENT_SHADER)?;
        let program = Self::link(vertex, fragment)?;

        unsafe {
            gl::DeleteShader(vertex);
            gl::DeleteShader(fragment);
        }

        Ok(Self { program })
    }

    fn compile(source: &str, kind: u32) -> Result<u32, ShaderError> {
        let shader = unsafe { gl::CreateShader(kind) };

        unsafe {
            let sources = [source.as_ptr().cast::<i8>()];
            let lengths = [source.len() as i32];

            gl::ShaderSource(shader, 1, sources.as_ptr(), lengths.as_ptr());
            gl::CompileShader(shader);
        }

        let mut success = 0;
        unsafe {
            gl::GetShaderiv(shader, gl::COMPILE_STATUS, &raw mut success);
        }

        if success == 1 {
            Ok(shader)
        } else {
            let mut log_length = 0;
            unsafe {
                gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &raw mut log_length);
            }

            let mut buffer = vec![0u8; log_length as usize];
            unsafe {
                gl::GetShaderInfoLog(
                    shader,
                    log_length,
                    std::ptr::null_mut(),
                    buffer.as_mut_ptr().cast::<i8>(),
                );
            }

            let error_log = String::from_utf8_lossy(&buffer);
            unsafe {
                gl::DeleteShader(shader);
            }

            let shader_type_str = match kind {
                gl::VERTEX_SHADER => "vertex",
                gl::FRAGMENT_SHADER => "fragment",
                _ => "unknown",
            };

            Err(ShaderError::Compile(format!(
                "Failed to compile {shader_type_str} shader: {error_log}"
            )))
        }
    }

    fn link(vertex: u32, fragment: u32) -> Result<u32, ShaderError> {
        let program = unsafe { gl::CreateProgram() };

        unsafe {
            gl::AttachShader(program, vertex);
            gl::AttachShader(program, fragment);
            gl::LinkProgram(program);

            gl::DetachShader(program, vertex);
            gl::DetachShader(program, fragment);
        }

        let mut success = 0;
        unsafe {
            gl::GetProgramiv(program, gl::LINK_STATUS, &raw mut success);
        }

        if success == 1 {
            Ok(program)
        } else {
            let mut log_length = 0;
            unsafe {
                gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &raw mut log_length);
            }

            let mut buffer = vec![0u8; log_length as usize];
            unsafe {
                gl::GetProgramInfoLog(
                    program,
                    log_length,
                    std::ptr::null_mut(),
                    buffer.as_mut_ptr().cast::<i8>(),
                );
            }

            let error_log = String::from_utf8_lossy(&buffer);
            unsafe {
                gl::DeleteProgram(program);
            }

            Err(ShaderError::Linking(format!(
                "Failed to link shader program: {error_log}"
            )))
        }
    }

    pub fn bind(&self) {
        unsafe {
            gl::UseProgram(self.program);
        }
    }

    pub fn unbind() {
        unsafe {
            gl::UseProgram(0);
        }
    }

    fn attribute_location(&self, name: &str) -> Option<i32> {
        let cname = CString::new(name).ok()?;
        let location = unsafe { gl::GetAttribLocation(self.program, cname.as_ptr()) };

        if location >= 0 { Some(location) } else { None }
    }

    pub fn set_attribute(
        &self,
        name: &str,
        index: u32,
        stride: i32,
        offset: u32,
        attribute: VertexAttribute,
    ) {
        let Some(location) = self.attribute_location(name) else {
            return;
        };

        unsafe {
            gl::VertexAttribPointer(
                index,
                attribute.gl_size(),
                attribute.gl_type(),
                gl::FALSE,
                stride,
                offset as *const _,
            );
        }
    }

    fn uniform_location(&self, name: &str) -> Option<i32> {
        let cname = CString::new(name).ok()?;
        let location = unsafe { gl::GetUniformLocation(self.program, cname.as_ptr()) };

        if location >= 0 { Some(location) } else { None }
    }

    pub fn set_uniform(&self, name: &str, uniform: &Uniform) {
        if let Some(location) = self.uniform_location(name) {
            uniform.set(location);
        }
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteProgram(self.program);
        }
    }
}

pub(crate) enum Uniform {
    Float(f32),
    Vec2(Vec2),
    Vec3(Vec3),
    Vec4(Vec4),
    Mat4(Mat4),
}

impl Uniform {
    fn set(&self, location: i32) {
        unsafe {
            match self {
                Self::Float(float) => gl::Uniform1f(location, *float),
                Self::Vec2(vec) => gl::Uniform2f(location, vec.x, vec.y),
                Self::Vec3(vec) => gl::Uniform3f(location, vec.x, vec.y, vec.z),
                Self::Vec4(vec) => gl::Uniform4f(location, vec.x, vec.y, vec.z, vec.w),
                Self::Mat4(mat) => {
                    gl::UniformMatrix4fv(location, 1, gl::FALSE, mat.as_ref().as_ptr());
                }
            }
        }
    }
}

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

    fn gl_type(&self) -> u32 {
        match self {
            Self::Float | Self::Vec2 | Self::Vec3 | Self::Vec4 => gl::FLOAT,
        }
    }
}
