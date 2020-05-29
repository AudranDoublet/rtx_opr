extern crate gl;
extern crate glutin;

use crate::errors::*;
use crate::glchk_stmt;
use crate::helper;
use crate::CubeTracerArguments;

pub struct CubeTracer {
    program_raytracer: u32,
    program_quad_screen: u32,
    vao_quad_screen: u32,
    texture_raytracer: u32,

    pub args: CubeTracerArguments,
}

impl CubeTracer {
    pub fn new(
        width: u32,
        height: u32,
        view_size: usize,
        shadow_activated: bool,
    ) -> Result<Self, GLError> {
        let prog_raytracer_id = helper::build_program_raytracer(view_size, shadow_activated)?;
        let prog_quad_screen_id = helper::build_program_quad()?;

        let program_raytracer = prog_raytracer_id;
        let program_quad_screen = prog_quad_screen_id;

        let vao_quad_screen = helper::make_quad_vao(prog_quad_screen_id)?;
        let texture_raytracer = helper::generate_texture(width, height)?;

        helper::texture_3d(1, vec![
            &std::path::Path::new("data/stone.png"),
            &std::path::Path::new("data/stone_n.png"),
            &std::path::Path::new("data/dirt.png"),
            &std::path::Path::new("data/dirt_n.png"),
            &std::path::Path::new("data/grass_top.png"),
            &std::path::Path::new("data/grass_top_n.png"),
            &std::path::Path::new("data/grass_side.png"),
            &std::path::Path::new("data/grass_side_n.png"),
            &std::path::Path::new("data/sand.png"),
            &std::path::Path::new("data/sand_n.png"),
            &std::path::Path::new("data/snow.png"),
            &std::path::Path::new("data/snow_n.png"),
            &std::path::Path::new("data/gravel.png"),
            &std::path::Path::new("data/gravel_n.png"),
            &std::path::Path::new("data/cactus_top.png"),
            &std::path::Path::new("data/cactus_top_n.png"),
            &std::path::Path::new("data/cactus_side.png"),
            &std::path::Path::new("data/cactus_side_n.png"),
            &std::path::Path::new("data/cactus_bottom.png"),
            &std::path::Path::new("data/cactus_bottom_n.png"),
            &std::path::Path::new("data/log_oak.png"),
            &std::path::Path::new("data/log_oak_n.png"),
            &std::path::Path::new("data/log_acacia.png"),
            &std::path::Path::new("data/log_acacia_n.png"),
            &std::path::Path::new("data/log_big_oak.png"),
            &std::path::Path::new("data/log_big_oak_n.png"),
            &std::path::Path::new("data/log_birch.png"),
            &std::path::Path::new("data/log_birch_n.png"),
            &std::path::Path::new("data/log_jungle.png"),
            &std::path::Path::new("data/log_jungle_n.png"),
            &std::path::Path::new("data/log_spruce.png"),
            &std::path::Path::new("data/log_spruce_n.png"),
            &std::path::Path::new("data/log_oak_top.png"),
            &std::path::Path::new("data/log_oak_top_n.png"),
            &std::path::Path::new("data/log_acacia_top.png"),
            &std::path::Path::new("data/log_acacia_top_n.png"),
            &std::path::Path::new("data/log_big_oak_top.png"),
            &std::path::Path::new("data/log_big_oak_top_n.png"),
            &std::path::Path::new("data/log_birch_top.png"),
            &std::path::Path::new("data/log_birch_top_n.png"),
            &std::path::Path::new("data/log_jungle_top.png"),
            &std::path::Path::new("data/log_jungle_top_n.png"),
            &std::path::Path::new("data/log_spruce_top.png"),
            &std::path::Path::new("data/log_spruce_top_n.png"),
        ])?;

        Ok(CubeTracer {
            program_raytracer,
            program_quad_screen,
            vao_quad_screen,
            texture_raytracer,

            args: CubeTracerArguments::new(program_raytracer, view_size)?,
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
