extern crate gl;
extern crate glutin;

use crate::errors::*;
use crate::glchk_stmt;
use crate::helper;
use crate::CubeTracerArguments;

#[inline]
fn coeff(i: u32, coeff: f32) -> u32 {
    (i as f32 / coeff) as u32
}

pub struct CubeTracer {
    program_raytracer: u32,
    program_quad_screen: u32,
    vao_quad_screen: u32,
    vao_quad_cursor: u32,
    texture_raytracer: u32,
    texture_cursor: u32,
    resolution_coeff: f32,

    cache_intersections: u32,
    cache_normals: u32,

    pub args: CubeTracerArguments,
}

impl CubeTracer {
    pub fn new(
        width: u32,
        height: u32,
        view_size: usize,
        resolution_coeff: f32,
        shadow_activated: bool,
    ) -> Result<Self, GLError> {
        let prog_raytracer_id = helper::build_program_raytracer(view_size, shadow_activated, coeff(10, resolution_coeff))?;
        let prog_quad_screen_id = helper::build_program_quad()?;

        let program_raytracer = prog_raytracer_id;
        let program_quad_screen = prog_quad_screen_id;

        let vao_quad_screen = helper::make_quad_vao(prog_quad_screen_id, 1.0, 1.0)?;
        let vao_quad_cursor = helper::make_quad_vao(prog_quad_screen_id, 0.01125, 0.02)?;

        let width = coeff(width, resolution_coeff);
        let height = coeff(height, resolution_coeff);

        let texture_raytracer = helper::generate_texture(width, height)?;

        let cache_intersections= helper::generate_image_cache(1, width, height)?;
        let cache_normals = helper::generate_image_cache(2, width, height)?;

        helper::texture_3d(
            1,
            vec![
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
                &std::path::Path::new("data/tallgrass.png"),
                &std::path::Path::new("data/tallgrass_n.png"),
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
                &std::path::Path::new("data/leaves_oak.png"),
                &std::path::Path::new("data/leaves_oak_n.png"),
                &std::path::Path::new("data/leaves_acacia.png"),
                &std::path::Path::new("data/leaves_acacia_n.png"),
                &std::path::Path::new("data/leaves_big_oak.png"),
                &std::path::Path::new("data/leaves_big_oak_n.png"),
                &std::path::Path::new("data/leaves_birch.png"),
                &std::path::Path::new("data/leaves_birch_n.png"),
                &std::path::Path::new("data/leaves_jungle.png"),
                &std::path::Path::new("data/leaves_jungle_n.png"),
                &std::path::Path::new("data/leaves_spruce.png"),
                &std::path::Path::new("data/leaves_spruce_n.png"),
                &std::path::Path::new("data/flower_tulip_orange.png"),
                &std::path::Path::new("data/flower_tulip_orange_n.png"),
                &std::path::Path::new("data/flower_tulip_pink.png"),
                &std::path::Path::new("data/flower_tulip_pink_n.png"),
                &std::path::Path::new("data/flower_tulip_red.png"),
                &std::path::Path::new("data/flower_tulip_red_n.png"),
                &std::path::Path::new("data/flower_tulip_white.png"),
                &std::path::Path::new("data/flower_tulip_white_n.png"),
                &std::path::Path::new("data/flower_dandelion.png"),
                &std::path::Path::new("data/flower_dandelion_n.png"),
                &std::path::Path::new("data/flower_houstonia.png"),
                &std::path::Path::new("data/flower_houstonia_n.png"),
                &std::path::Path::new("data/flower_oxeye_daisy.png"),
                &std::path::Path::new("data/flower_oxeye_daisy_n.png"),
                &std::path::Path::new("data/flower_blue_orchid.png"),
                &std::path::Path::new("data/flower_blue_orchid_n.png"),
                &std::path::Path::new("data/flower_allium.png"),
                &std::path::Path::new("data/flower_allium_n.png"),
                &std::path::Path::new("data/flower_rose.png"),
                &std::path::Path::new("data/flower_rose_n.png"),
                &std::path::Path::new("data/grass_side_overlay.png"),
                &std::path::Path::new("data/grass_side_overlay_n.png"),
            ],
        )?;

        let texture_cursor = helper::load_texture(2, &std::path::Path::new("data/cursor.png"))?;

        Ok(CubeTracer {
            program_raytracer,
            program_quad_screen,
            vao_quad_screen,
            vao_quad_cursor,
            texture_raytracer,
            texture_cursor,
            resolution_coeff,

            cache_intersections,
            cache_normals,

            args: CubeTracerArguments::new(program_raytracer, view_size)?,
        })
    }

    pub fn compute_image(&self, width: u32, height: u32) -> Result<(), GLError> {
        let width = coeff(width, self.resolution_coeff);
        let height = coeff(height, self.resolution_coeff);

        glchk_stmt!(
            gl::UseProgram(self.program_raytracer);

            gl::DispatchCompute((width + 7) / 8, (height + 7) / 8, 1);
            gl::MemoryBarrier(gl::SHADER_IMAGE_ACCESS_BARRIER_BIT);
        );

        Ok(())
    }

    pub fn resize(&mut self, width: u32, height: u32) -> Result<(), GLError> {
        glchk_stmt!(
            gl::DeleteTextures(3,
                [self.texture_raytracer, self.cache_intersections, self.cache_normals].as_ptr());
            gl::Viewport(
                0,
                0,
                width as i32,
                height as i32,
            );
        );

        let (w, h) = (coeff(width, self.resolution_coeff), coeff(height, self.resolution_coeff));
        self.texture_raytracer = helper::generate_texture(w, h)?;
        self.cache_intersections = helper::generate_image_cache(1, w, h)?;
        self.cache_normals = helper::generate_image_cache(2, w, h)?;

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

            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);

            gl::BindVertexArray(self.vao_quad_cursor);
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, self.texture_cursor);
            gl::DrawArrays(gl::TRIANGLE_STRIP, 0, 4);

            gl::Disable(gl::BLEND);
        );

        Ok(())
    }
}
