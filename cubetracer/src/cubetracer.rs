extern crate gl;
extern crate glutin;

use crate::errors::*;
use crate::glchk_stmt;
use crate::helper;

// Correspondence Table indexes
const GL_PROG_RAY_VAR_IDX_ROLL: usize = 0;
const GL_PROG_RAY_VAR_LEN: usize = 1;

pub struct CubeTracerArguments {
    program: u32,
    uniform_locations: [i32; GL_PROG_RAY_VAR_LEN],
}

impl CubeTracerArguments {
    pub fn new(program: u32) -> Result<Self, GLError> {
        let mut uniform_locations = [-1; GL_PROG_RAY_VAR_LEN];

        uniform_locations[GL_PROG_RAY_VAR_IDX_ROLL] =
            helper::get_uniform_location(program, "roll")?;

        Ok(CubeTracerArguments {
            program,
            uniform_locations,
        })
    }

    pub fn set_roll(&self, value: f32) -> Result<(), GLError> {
        glchk_stmt!(
            gl::ProgramUniform1f(
                self.program,
                self.uniform_locations[GL_PROG_RAY_VAR_IDX_ROLL],
                value,
            );
        );

        Ok(())
    }
}

pub struct CubeTracer {
    program_raytracer: u32,
    program_quad_screen: u32,
    vao_quad_screen: u32,
    texture_raytracer: u32,

    pub args: CubeTracerArguments,
}

impl CubeTracer {
    pub fn new(width: u32, height: u32) -> Result<Self, GLError> {
        let prog_raytracer_id = helper::build_program_raytracer()?;
        let prog_quad_screen_id = helper::build_program_quad()?;

        let program_raytracer = prog_raytracer_id;
        let program_quad_screen = prog_quad_screen_id;

        let vao_quad_screen = helper::make_quad_vao(prog_quad_screen_id)?;
        let texture_raytracer = helper::generate_texture(width, height)?;

        Ok(CubeTracer {
            program_raytracer,
            program_quad_screen,
            vao_quad_screen,
            texture_raytracer,

            args: CubeTracerArguments::new(program_raytracer)?,
        })
    }

    pub fn compute_image(&self, width: u32, height: u32) -> Result<(), GLError> {
        glchk_stmt!(
            gl::UseProgram(self.program_raytracer);

            gl::DispatchCompute((width + 31) / 32, (height + 31) / 32, 1);
            gl::MemoryBarrier(gl::SHADER_IMAGE_ACCESS_BARRIER_BIT);
        );

        Ok(())
    }

    pub fn resize(&mut self, width: u32, height: u32) -> Result<(), GLError> {
        glchk_stmt!(
            gl::DeleteTextures(1, &self.texture_raytracer);
            gl::Viewport(
                0,
                0,
                width as i32,
                height as i32,
            );
        );

        self.texture_raytracer = helper::generate_texture(width, height)?;

        Ok(())
    }

    pub fn draw(&self) -> Result<(), GLError> {
        glchk_stmt!(
            gl::Clear(gl::COLOR_BUFFER_BIT);

            gl::UseProgram(self.program_quad_screen);
            gl::BindVertexArray(self.vao_quad_screen);
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, self.texture_raytracer);
            gl::DrawArrays(gl::TRIANGLE_STRIP, 0, 4);
        );

        Ok(())
    }
}
