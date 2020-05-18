extern crate gl;

use crate::errors::*;
use crate::glchk_stmt;
use crate::helper;
use crate::Camera;

use nalgebra::{Vector2, Vector3};

use world::{Block, Chunk};

// Correspondence Table indexes
const VAR_IDX_SCREEN_DOT_TOP_LEFT: usize = 0;
const VAR_IDX_SCREEN_DOT_LEFT: usize = 1;
const VAR_IDX_SCREEN_DOT_UP: usize = 2;
const VAR_IDX_ORIGIN: usize = 3;
const VAR_IDX_CL_MIN_COORDS: usize = 4;
const VARS_LEN: usize = 5;

pub struct CubeTracerArguments {
    program: u32,
    ssbo_raytracer_cl_filled: u32,
    ssbo_raytracer_cl_blocks: u32,
    view_size: usize,
    uniform_locations: [i32; VARS_LEN],
}

impl CubeTracerArguments {
    pub fn new(program: u32, view_size: usize) -> Result<Self, GLError> {
        let mut uniform_locations = [-1; VARS_LEN];

        let nb_chunks = (2 * view_size).pow(2);
        let ssbo_raytracer_cl_filled =
            helper::make_ssbo(program, "in_cl_data_0", &vec![true; nb_chunks * 16])?;
        let ssbo_raytracer_cl_blocks = helper::make_ssbo(
            program,
            "in_cl_data_1",
            &vec![0 as u32; nb_chunks * 16 * 16 * 256],
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

        Ok(CubeTracerArguments {
            program,
            ssbo_raytracer_cl_filled,
            ssbo_raytracer_cl_blocks,
            view_size,
            uniform_locations,
        })
    }

    pub fn set_chunks(&self, mut chunks: Vec<&Box<Chunk>>) -> Result<(), GLError> {
        let nb_chunks_x = 2 * self.view_size;
        let nb_chunks_xz = nb_chunks_x.pow(2);

        let mut chunks_filled = vec![[0 as u32; 16]; nb_chunks_xz];
        let mut chunks_blocks = vec![[Block::Air as u32; 16 * 16 * 256]; nb_chunks_xz];

        chunks.sort_unstable();

        let mut cl_min_coords = Vector2::new(std::i32::MAX, std::i32::MAX);
        chunks.iter().map(|c| c.coords()).for_each(|c| {
            cl_min_coords.x = cl_min_coords.x.min(c.x);
            cl_min_coords.y = cl_min_coords.y.min(c.y);
        });

        chunks.iter().for_each(|_c| {
            /*
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
            */
        });

        helper::update_ssbo_data(self.ssbo_raytracer_cl_blocks, &chunks_blocks)?;
        helper::update_ssbo_data(self.ssbo_raytracer_cl_filled, &chunks_filled)?;

        self.set_vector_3i(
            VAR_IDX_CL_MIN_COORDS,
            Vector3::new(cl_min_coords.x, 0, cl_min_coords.y),
        )?;

        Ok(())
    }

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

    pub fn set_camera(&self, value: &Camera) -> Result<(), GLError> {
        let origin = value.origin;
        let top_left = value.get_virtual_screen_top_left();
        let (left, up) = value.get_virtual_screen_axes_scaled();

        // FIXME: we should try to send the data as an array of 4 Vector3 in one shot
        self.set_vector_3f(VAR_IDX_ORIGIN, origin)?;
        self.set_vector_3f(VAR_IDX_SCREEN_DOT_TOP_LEFT, top_left)?;
        self.set_vector_3f(VAR_IDX_SCREEN_DOT_LEFT, left)?;
        self.set_vector_3f(VAR_IDX_SCREEN_DOT_UP, up)?;
        // FIXME-END

        Ok(())
    }
}
