extern crate gl;

use crate::errors::*;
use crate::glchk_stmt;
use crate::helper;
use crate::Camera;

use nalgebra::{Vector2, Vector3};
use std::{mem, rc::Rc};

use world::{Block, Chunk};

// Correspondence Table indexes
const VAR_IDX_SCREEN_DOT_TOP_LEFT: usize = 0;
const VAR_IDX_SCREEN_DOT_LEFT: usize = 1;
const VAR_IDX_SCREEN_DOT_UP: usize = 2;
const VAR_IDX_ORIGIN: usize = 3;
const VAR_IDX_CL_MIN_COORDS: usize = 4;
const VAR_IDX_HIGHTLIGHTED_BLOCK: usize = 5;
const VAR_IDX_TEXTURES : usize = 6;

const VARS_LEN: usize = 8;

pub struct CubeTracerArguments {
    program: u32,
    ssbo_raytracer_cl: u32,
    view_size: usize,
    uniform_locations: [i32; VARS_LEN],
}

impl CubeTracerArguments {
    pub fn new(program: u32, view_size: usize) -> Result<Self, GLError> {
        let mut uniform_locations = [-1; VARS_LEN];

        let nb_chunks = (2 * view_size).pow(2);
        let cl_nb_blocks = 16 * 16 * 256;
        let cl_nb_filled = 16;

        let ssbo_raytracer_cl = helper::make_ssbo(
            program,
            "shader_data",
            nb_chunks * (cl_nb_filled + cl_nb_blocks) * mem::size_of::<u32>(),
        )?;

        // Camera variables
        uniform_locations[VAR_IDX_SCREEN_DOT_TOP_LEFT] =
            helper::get_uniform_location(program, "in_uni_screen.top_left")?;
        uniform_locations[VAR_IDX_SCREEN_DOT_LEFT] =
            helper::get_uniform_location(program, "in_uni_screen.left")?;
        uniform_locations[VAR_IDX_SCREEN_DOT_UP] =
            helper::get_uniform_location(program, "in_uni_screen.up")?;
        uniform_locations[VAR_IDX_ORIGIN] = helper::get_uniform_location(program, "in_uni_origin")?;

        // Chunk Loads variables
        uniform_locations[VAR_IDX_CL_MIN_COORDS] =
            helper::get_uniform_location(program, "in_uni_cl_min_coords")?;

        // Hightlighted block variables
        uniform_locations[VAR_IDX_HIGHTLIGHTED_BLOCK] =
            helper::get_uniform_location(program, "in_uni_highlighted_block")?;

        uniform_locations[VAR_IDX_TEXTURES] =
            helper::get_uniform_location(program, "in_uni_textures")?;

        let res = CubeTracerArguments {
            program,
            ssbo_raytracer_cl,
            view_size,
            uniform_locations,
        };

        res.set_i(VAR_IDX_TEXTURES, 1)?;
        Ok(res)
    }

    pub fn set_chunks(&self, mut chunks: Vec<Rc<Chunk>>) -> Result<Vector2<i32>, GLError> {
        let nb_chunks_x = 2 * self.view_size;
        let nb_chunks_xz = nb_chunks_x.pow(2);

        assert!(chunks.len() <= nb_chunks_xz as usize);
        chunks.sort_unstable();

        let mut chunks_blocks = vec![[Block::Air as u32; 16 * 16 * 256]; nb_chunks_xz];
        let mut chunks_filled = vec![[false as u32; 16]; nb_chunks_xz];

        let mut cl_min_coords = Vector2::new(std::i32::MAX, std::i32::MAX);
        chunks.iter().map(|c| c.coords()).for_each(|c| {
            cl_min_coords.x = cl_min_coords.x.min(c.x);
            cl_min_coords.y = cl_min_coords.y.min(c.y);
        });

        chunks.iter().for_each(|c| {
            let coord = c.coords() - cl_min_coords;
            let coord = (coord.x as usize, coord.y as usize);

            let xz = coord.0 + coord.1 * nb_chunks_x;

            // -- Fill blocks --
            c.blocks.iter().enumerate().for_each(|(i, &b)| {
                chunks_blocks[xz][i] = b as u32;
            });

            let chunks_filled_cur = &mut chunks_filled[xz];
            c.chunk_filled_metadata()
                .iter()
                .enumerate()
                .for_each(|(y, &filled)| {
                    chunks_filled_cur[y] = filled as u32;
                });
        });

        helper::update_ssbo_partial(self.ssbo_raytracer_cl, 0, &chunks_filled)?;
        helper::update_ssbo_partial(
            self.ssbo_raytracer_cl,
            16 * chunks_filled.len() * mem::size_of::<u32>(),
            &chunks_blocks,
        )?;

        /*
        helper::update_ssbo_data(self.ssbo_raytracer_cl_blocks, &chunks_blocks)?;
        helper::update_ssbo_data(self.ssbo_raytracer_cl_filled, &chunks_filled)?;
        */

        self.set_vector_2i(VAR_IDX_CL_MIN_COORDS, cl_min_coords)?;

        Ok(cl_min_coords)
    }

    fn set_i(&self, var_idx: usize, value: i32) -> Result<(), GLError> {
        glchk_stmt!(
            gl::ProgramUniform1iv(
                self.program,
                self.uniform_locations[var_idx],
                1,
                (&value) as *const _
            );
        );

        Ok(())
    }

    fn set_vector_2i(&self, var_idx: usize, value: Vector2<i32>) -> Result<(), GLError> {
        glchk_stmt!(
            gl::ProgramUniform2iv(
                self.program,
                self.uniform_locations[var_idx],
                1,
                value.data.as_ptr(),
            );
        );

        Ok(())
    }

    #[allow(dead_code)]
    fn set_vector_3i(&self, var_idx: usize, value: Vector3<i32>) -> Result<(), GLError> {
        glchk_stmt!(
            gl::ProgramUniform3iv(
                self.program,
                self.uniform_locations[var_idx],
                1,
                value.data.as_ptr(),
            );
        );

        Ok(())
    }

    fn set_vector_3f(&self, var_idx: usize, value: Vector3<f32>) -> Result<(), GLError> {
        glchk_stmt!(
            gl::ProgramUniform3fv(
                self.program,
                self.uniform_locations[var_idx],
                1,
                value.data.as_ptr(),
            );
        );

        Ok(())
    }

    pub fn set_camera(&self, value: &Camera, highlighted_block: Vector3<i32>) -> Result<(), GLError> {
        let origin = value.origin;
        let top_left = value.get_virtual_screen_top_left();
        let (left, up) = value.get_virtual_screen_axes_scaled();

        // FIXME: we should try to send the data as an array of 4 Vector3 in one shot
        self.set_vector_3f(VAR_IDX_ORIGIN, origin)?;
        self.set_vector_3f(VAR_IDX_SCREEN_DOT_TOP_LEFT, top_left)?;
        self.set_vector_3f(VAR_IDX_SCREEN_DOT_LEFT, left)?;
        self.set_vector_3f(VAR_IDX_SCREEN_DOT_UP, up)?;
        self.set_vector_3i(VAR_IDX_HIGHTLIGHTED_BLOCK, highlighted_block)?;
        // FIXME-END

        Ok(())
    }
}
