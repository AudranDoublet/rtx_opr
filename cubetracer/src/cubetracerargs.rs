extern crate gl;

use crate::errors::*;
use crate::glchk_stmt;
use crate::helper;
use crate::Camera;

use nalgebra::{Vector2, Vector3};
use std::{collections::HashMap, mem, rc::Rc};

use world::Chunk;

// Correspondence Table indexes
const VAR_IDX_SCREEN_DOT_TOP_LEFT: usize = 0;
const VAR_IDX_SCREEN_DOT_LEFT: usize = 1;
const VAR_IDX_SCREEN_DOT_UP: usize = 2;
const VAR_IDX_ORIGIN: usize = 3;
const VAR_IDX_CL_MIN_COORDS: usize = 4;
const VAR_IDX_HIGHTLIGHTED_BLOCK: usize = 5;
const VAR_IDX_TEXTURES: usize = 6;
const VAR_IDX_WIND: usize = 7;
const VAR_IDX_ITERATION_ID: usize = 8;

const VARS_LEN: usize = 9;

pub struct CubeTracerArguments {
    program: u32,
    ssbo_raytracer_cl: u32,
    view_size: usize,
    chunks_available_indices: Vec<u32>,
    chunks_mapping: HashMap<(i32, i32), u32>,
    cl_min_coords: Vector3<i32>,
    uniform_locations: [i32; VARS_LEN],
    iteration_id: i32,
}

fn chunk_size() -> usize {
    16*16*256 + 16*16*3
}

impl CubeTracerArguments {
    pub fn new(program: u32, view_size: usize) -> Result<Self, GLError> {
        let mut uniform_locations = [-1; VARS_LEN];

        let nb_chunks = (2 * view_size).pow(2);
        let cl_nb_mapping = 1;
        let cl_nb_blocks = chunk_size();

        let ssbo_raytracer_cl = helper::make_ssbo(
            program,
            "shader_data",
            (nb_chunks + 1) * (cl_nb_mapping + cl_nb_blocks) * mem::size_of::<u32>(),
        )?;

        // init the empty chunk (used during data stream)
        helper::update_ssbo_partial(
            ssbo_raytracer_cl,
            (nb_chunks + nb_chunks * cl_nb_blocks) * mem::size_of::<u32>(),
            &vec![0 as u32; cl_nb_blocks],
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

        uniform_locations[VAR_IDX_ITERATION_ID] =
            helper::get_uniform_location(program, "in_uni_iteration_id")?;

        uniform_locations[VAR_IDX_WIND] = helper::get_uniform_location(program, "in_uni_wind")?;

        let res = CubeTracerArguments {
            program,
            ssbo_raytracer_cl,
            view_size,
            chunks_available_indices: (0..nb_chunks as u32).collect(),
            chunks_mapping: HashMap::new(),
            cl_min_coords: Vector3::zeros(),
            uniform_locations,
            iteration_id: 0,
        };

        res.set_i(VAR_IDX_TEXTURES, 1)?;
        Ok(res)
    }

    pub fn nb_mapped_chunks(&self) -> usize {
        self.chunks_mapping.len()
    }

    pub fn update_chunks(
        &mut self,
        chunks_to_remove: Vec<(i32, i32)>,
        mut chunks_to_add: Vec<Rc<Chunk>>,
    ) -> Result<Vector2<i32>, GLError> {
        let nb_chunks_x = 2 * self.view_size;
        let nb_chunks_xz = nb_chunks_x.pow(2);

        let mut chunks_mapping = vec![(nb_chunks_xz + 1) as u32; nb_chunks_xz];
        let mem_chunk_size = chunk_size() * mem::size_of::<u32>();
        let mem_blocks_offset = chunks_mapping.len() * mem::size_of::<u32>();

        for chunk_to_rm in chunks_to_remove {
            if let Some(idx) = self.chunks_mapping.remove(&chunk_to_rm) {
                self.chunks_available_indices.push(idx);
            }
        }

        while let Some(chunk_to_add) = chunks_to_add.pop() {
            let coords = chunk_to_add.coords();
            let (x, y) = (coords.x, coords.y);

            let mut chunk_to_add_data = chunk_to_add
                .blocks
                .iter()
                .map(|&b| b as u32)
                .collect::<Vec<u32>>();

            chunk_to_add_data.extend(
                chunk_to_add.grass_color
                    .iter().map(|&b| unsafe {
                        std::mem::transmute::<f32, u32>(b)
                    })
            );

            let chunk_idx = if let Some(idx) = self.chunks_mapping.get(&(x, y)) {
                *idx
            } else {
                self.chunks_available_indices.pop().unwrap()
            } as usize;

            self.chunks_mapping.insert((x, y), chunk_idx as u32);

            helper::update_ssbo_partial(
                self.ssbo_raytracer_cl,
                mem_blocks_offset + chunk_idx * mem_chunk_size,
                &chunk_to_add_data,
            )?;
        }

        // FIXME: we lazily update the cl_min_coords instead of iterating through all chunks on
        // every updates
        let mut cl_min_coords = Vector2::new(std::i32::MAX, std::i32::MAX);
        self.chunks_mapping.keys().for_each(|&(x, y)| {
            cl_min_coords.x = cl_min_coords.x.min(x);
            cl_min_coords.y = cl_min_coords.y.min(y);
        });
        self.cl_min_coords = Vector3::new(cl_min_coords.x, 0, cl_min_coords.y);
        // FIXME-END

        self.chunks_mapping.iter().for_each(|(&(x, y), &idx)| {
            let coord = Vector2::new(x, y) - cl_min_coords;
            let coord = (coord.x as usize, coord.y as usize);

            let xz = coord.0 + coord.1 * nb_chunks_x;

            // -- Fill blocks --
            chunks_mapping[xz] = idx as u32;
        });

        helper::update_ssbo_partial(self.ssbo_raytracer_cl, 0, &chunks_mapping)?;
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

    pub fn set_camera(
        &mut self,
        change: bool,
        value: &Camera,
        wind: Vector3<f32>,
        mut highlighted_block: Vector3<i32>,
    ) -> Result<(), GLError> {
        if !change {
            self.iteration_id = 0; //FIXME += 1
            self.set_i(VAR_IDX_ITERATION_ID, self.iteration_id)?;
        } else {
            self.iteration_id = 0;

            let origin = value.origin;
            let top_left = value.get_virtual_screen_top_left();
            let (left, up) = value.get_virtual_screen_axes_scaled();

            highlighted_block -= self.cl_min_coords.component_mul(&Vector3::new(16, 0, 16));

            // FIXME: we should try to send the data as an array of 4 Vector3 in one shot
            self.set_vector_3f(VAR_IDX_ORIGIN, origin)?;
            self.set_vector_3f(VAR_IDX_SCREEN_DOT_TOP_LEFT, top_left)?;
            self.set_vector_3f(VAR_IDX_SCREEN_DOT_LEFT, left)?;
            self.set_vector_3f(VAR_IDX_SCREEN_DOT_UP, up)?;
            self.set_vector_3i(VAR_IDX_HIGHTLIGHTED_BLOCK, highlighted_block)?;
            self.set_i(VAR_IDX_ITERATION_ID, self.iteration_id)?;
            // FIXME-END
        }

        self.set_vector_3f(VAR_IDX_WIND, wind)?;

        Ok(())
    }
}
