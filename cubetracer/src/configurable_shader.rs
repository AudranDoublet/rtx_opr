extern crate gl;

use std::ffi::CString;

use crate::errors::*;
use crate::{glchk_expr, glchk_stmt};

use std::ptr;
use std::collections::HashMap;

pub struct ConfigurableShader
{
    shader: String,
}

impl ConfigurableShader {
    pub fn new(s: &str) -> ConfigurableShader {
        ConfigurableShader {
            shader: s.to_string(),
        }
    }

    pub fn var<T: std::fmt::Display>(&mut self, name: &str, value: T) -> &mut ConfigurableShader {
        let str_value = format!("<<<{}>>>", value);
        self.shader = self.shader.replace(name, &str_value);

        self
    }

    pub fn vars<T: std::fmt::Display>(&mut self, vars: &HashMap<&str, T>) {
        for (k, v) in vars {
            self.var(k, v);
        }
    }

    pub fn build(&self, shader_type: u32) -> Result<u32, GLError> {
        let shader = glchk_expr!(gl::CreateShader(shader_type));
        let c_str = CString::new(self.shader.as_bytes()).unwrap();

        glchk_stmt!(
            gl::ShaderSource(shader, 1, &c_str.as_ptr(), ptr::null());
            gl::CompileShader(shader);
        );

        gl_check_error_shader(shader, gl::COMPILE_STATUS)?;

        Ok(shader)
    }
}
