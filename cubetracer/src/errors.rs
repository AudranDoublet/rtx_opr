extern crate gl;

use gl::types::*;
use std::{fmt, str};

const GL_MAX_ERROR_LEN: usize = 512;
pub enum GLError {
    InvalidEnum,
    InvalidValue,
    InvalidOperation,
    StackUnderlow,
    StackOverflow,
    OutOfMemory,
    InvalidFrameBufferOperation,
    UnknownError,

    UniformNotFound { name: String },

    ProgramError { program: u32 },
    ShaderError { shader: u32 },
}

#[macro_export]
macro_rules! glchk_stmt {
    ($($s:stmt;)*) => {
        unsafe {
            $(
                $s
                let err = gl::GetError();
                (
                    if err != gl::NO_ERROR {
                        Err(match err {
                            gl::INVALID_ENUM => GLError::InvalidEnum,
                            gl::INVALID_VALUE => GLError::InvalidValue,
                            gl::INVALID_OPERATION => GLError::InvalidOperation,
                            gl::OUT_OF_MEMORY => GLError::OutOfMemory,
                            gl::STACK_UNDERFLOW => GLError::StackUnderlow,
                            gl::STACK_OVERFLOW => GLError::StackOverflow,
                            gl::INVALID_FRAMEBUFFER_OPERATION => GLError::InvalidFrameBufferOperation,
                            _ => GLError::UnknownError,
                        })
                    } else {
                        Ok(())
                    }
                )?;
            )*
        }
    }
}

#[macro_export]
macro_rules! glchk_expr {
    ($s:expr) => {
        unsafe {
            let r = $s;
            let err = gl::GetError();
            (if err != gl::NO_ERROR {
                Err(match err {
                    gl::INVALID_ENUM => GLError::InvalidEnum,
                    gl::INVALID_VALUE => GLError::InvalidValue,
                    gl::INVALID_OPERATION => GLError::InvalidOperation,
                    gl::OUT_OF_MEMORY => GLError::OutOfMemory,
                    gl::STACK_UNDERFLOW => GLError::StackUnderlow,
                    gl::STACK_OVERFLOW => GLError::StackOverflow,
                    gl::INVALID_FRAMEBUFFER_OPERATION => GLError::InvalidFrameBufferOperation,
                    _ => GLError::UnknownError,
                })
            } else {
                Ok((r))
            })?
        }
    };
}

fn fmt(err: &GLError, f: &mut fmt::Formatter) -> fmt::Result {
    match err {
        GLError::InvalidEnum => f.write_str("An unacceptable value is specified for an enumerated argument. 
                The offending command is ignored and has no other side effect than to set the error flag."),
        GLError::InvalidValue => f.write_str("A numeric argument is out of range. 
                The offending command is ignored and has no other side effect than to set the error flag."),
        GLError::InvalidOperation => f.write_str("The specified operation is not allowed in the current state. 
                The offending command is ignored and has no other side effect than to set the error flag."),
        GLError::OutOfMemory => f.write_str("There is not enough memory left to execute the command. 
                The state of the GL is undefined, except for the state of the error flags, after this error 
                is recorded."),
        GLError::StackUnderlow => f.write_str("Stack popping operation cannot be done because the stack is already 
                at its lowest point."),
        GLError::StackOverflow => f.write_str("Stack pushing operation cannot be done because it would overflow 
            the limit of that stack's size."),
        GLError::InvalidFrameBufferOperation => f.write_str("The command is trying to render to or read from the
                framebuffer while the currently bound framebuffer is not framebuffer complete (i.e. the 
                    return value from `glCheckFramebufferStatus` is not GL_FRAMEBUFFER_COMPLETE). 
                The offending command is ignored and has no other side effect than to set the error flag."),
        GLError::UnknownError => f.write_str("This is an unknown error, may the Force be with you..."),
        GLError::UniformNotFound { name } => f.write_fmt(format_args!(
                "The following uniform could not be found in the shader: `{}`.
                This either means that the variable really does not exist, or, that it has optimized and removed
                by the driver because it did not contribute directly/indirectly to the output of the shader.",
                name
        )),
        GLError::ProgramError {program} => {
            let mut err_buf = make_error_buffer(GL_MAX_ERROR_LEN);
            let mut err_length = 0;

            unsafe {gl::GetProgramInfoLog(*program, GL_MAX_ERROR_LEN as i32, &mut err_length, err_buf.as_mut_ptr() as *mut GLchar)};
            err_buf.resize(err_length as usize, 0);

            f.write_str(str::from_utf8(&err_buf).unwrap())
        },
        GLError::ShaderError {shader} => {
            let mut err_buf = make_error_buffer(GL_MAX_ERROR_LEN);
            let mut err_length = 0;

            unsafe {gl::GetShaderInfoLog(*shader, GL_MAX_ERROR_LEN as i32, &mut err_length, err_buf.as_mut_ptr() as *mut GLchar)};
            err_buf.resize(err_length as usize, 0);

            f.write_str(str::from_utf8(&err_buf).unwrap())
        },
    }
}

impl fmt::Debug for GLError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt(self, f)
    }
}

impl fmt::Display for GLError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt(self, f)
    }
}

fn make_error_buffer(capacity: usize) -> Vec<u8> {
    let mut info_log = Vec::with_capacity(capacity);
    unsafe { info_log.set_len(capacity - 1) };

    info_log
}

pub fn gl_check_error_shader(shader: u32, err_type: gl::types::GLenum) -> Result<u32, GLError> {
    let mut success = gl::FALSE as GLint;
    unsafe { gl::GetShaderiv(shader, err_type, &mut success) };

    if success != gl::TRUE as GLint {
        Err(GLError::ShaderError { shader })
    } else {
        Ok(shader)
    }
}

pub fn gl_check_error_program(program: u32, err_type: gl::types::GLenum) -> Result<u32, GLError> {
    let mut success = gl::FALSE as GLint;
    unsafe { gl::GetProgramiv(program, err_type, &mut success) };

    if success != gl::TRUE as GLint {
        Err(GLError::ProgramError { program })
    } else {
        Ok(program)
    }
}
