use std::ffi::CString;

use glam::{Mat4, Vec2, Vec3, Vec4};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ShaderError {
    #[error("Shader compilation error: {0}")]
    Compile(String),
    #[error("Shader linking error: {0}")]
    Linking(String),
}

pub(crate) struct Shader {
    program: u32,
}

impl Shader {
    pub fn load(vertex: &str, fragment: &str) -> anyhow::Result<Self> {
        let vertex = Self::compile(vertex, gl::VERTEX_SHADER)?;
        let fragment = Self::compile(fragment, gl::FRAGMENT_SHADER)?;
        let program = Self::link(vertex, fragment)?;

        unsafe {
            gl::DeleteShader(vertex);
            gl::DeleteShader(fragment);
        }

        Ok(Self { program })
    }

    fn compile(source: &str, kind: u32) -> anyhow::Result<u32> {
        let shader = unsafe { gl::CreateShader(kind) };

        unsafe {
            let sources = [source.as_ptr() as *const i8];
            let lengths = [source.len() as i32];

            gl::ShaderSource(shader, 1, sources.as_ptr(), lengths.as_ptr());
            gl::CompileShader(shader);
        }

        let mut success = 0;
        unsafe {
            gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut success);
        }

        if success == 1 {
            Ok(shader)
        } else {
            let mut log_length = 0;
            unsafe {
                gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut log_length);
            }

            let mut buffer = vec![0u8; log_length as usize];
            unsafe {
                gl::GetShaderInfoLog(
                    shader,
                    log_length,
                    std::ptr::null_mut(),
                    buffer.as_mut_ptr() as *mut i8,
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

            Err(anyhow::anyhow!(
                "Failed to compile {} shader: {}",
                shader_type_str,
                error_log
            ))
        }
    }

    fn link(vertex: u32, fragment: u32) -> anyhow::Result<u32> {
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
            gl::GetProgramiv(program, gl::LINK_STATUS, &mut success);
        }

        if success == 1 {
            Ok(program)
        } else {
            let mut log_length = 0;
            unsafe {
                gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut log_length);
            }

            let mut buffer = vec![0u8; log_length as usize];
            unsafe {
                gl::GetProgramInfoLog(
                    program,
                    log_length,
                    std::ptr::null_mut(),
                    buffer.as_mut_ptr() as *mut i8,
                );
            }

            let error_log = String::from_utf8_lossy(&buffer);
            unsafe {
                gl::DeleteProgram(program);
            }

            Err(anyhow::anyhow!(
                "Failed to link shader program: {}",
                error_log
            ))
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

    fn uniform_location(&self, name: &str) -> Option<i32> {
        let cname = CString::new(name).ok()?;
        let location = unsafe { gl::GetUniformLocation(self.program, cname.as_ptr()) };

        if location >= 0 { Some(location) } else { None }
    }

    pub fn set_f32(&self, name: &str, value: f32) {
        if let Some(loc) = self.uniform_location(name) {
            unsafe {
                gl::Uniform1f(loc, value);
            }
        }
    }

    pub fn set_i32(&self, name: &str, value: i32) {
        if let Some(loc) = self.uniform_location(name) {
            unsafe {
                gl::Uniform1i(loc, value);
            }
        }
    }

    pub fn set_bool(&self, name: &str, value: bool) {
        if let Some(loc) = self.uniform_location(name) {
            unsafe {
                gl::Uniform1i(loc, value as i32);
            }
        }
    }

    pub fn set_vec2(&self, name: &str, value: &Vec2) {
        if let Some(loc) = self.uniform_location(name) {
            unsafe {
                gl::Uniform2f(loc, value.x, value.y);
            }
        }
    }

    pub fn set_vec3(&self, name: &str, value: &Vec3) {
        if let Some(loc) = self.uniform_location(name) {
            unsafe {
                gl::Uniform3f(loc, value.x, value.y, value.z);
            }
        }
    }

    pub fn set_vec4(&self, name: &str, value: &Vec4) {
        if let Some(loc) = self.uniform_location(name) {
            unsafe {
                gl::Uniform4f(loc, value.x, value.y, value.z, value.w);
            }
        }
    }

    pub fn set_mat4(&self, name: &str, value: &Mat4) {
        if let Some(loc) = self.uniform_location(name) {
            unsafe {
                gl::UniformMatrix4fv(loc, 1, gl::FALSE, value.as_ref().as_ptr());
            }
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
