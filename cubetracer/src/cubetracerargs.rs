extern crate gl;

use crate::errors::*;
use crate::glchk_stmt;
use crate::helper;
use crate::Camera;

// Correspondence Table indexes
const VAR_IDX_SCREEN_DOT_TOP_LEFT: usize = 0;
const VAR_IDX_SCREEN_DOT_LEFT: usize = 1;
const VAR_IDX_SCREEN_DOT_UP: usize = 2;
const VAR_IDX_ORIGIN: usize = 3;
const VARS_LEN: usize = 4;

pub struct CubeTracerArguments {
    program: u32,
    uniform_locations: [i32; VARS_LEN],
}

impl CubeTracerArguments {
    pub fn new(program: u32) -> Result<Self, GLError> {
        let mut uniform_locations = [-1; VARS_LEN];

        uniform_locations[VAR_IDX_SCREEN_DOT_TOP_LEFT] =
            helper::get_uniform_location(program, "screen.top_left")?;

        uniform_locations[VAR_IDX_SCREEN_DOT_LEFT] =
            helper::get_uniform_location(program, "screen.left")?;

        uniform_locations[VAR_IDX_SCREEN_DOT_UP] =
            helper::get_uniform_location(program, "screen.up")?;

        uniform_locations[VAR_IDX_ORIGIN] = helper::get_uniform_location(program, "origin")?;

        Ok(CubeTracerArguments {
            program,
            uniform_locations,
        })
    }

    pub fn set_camera(&self, value: &Camera) -> Result<(), GLError> {
        let origin = value.origin;
        let top_left = value.get_virtual_screen_top_left();
        let (left, up) = value.get_virtual_screen_axes_scaled();

        // FIXME: we should try to send the data as an array of 4 Vector3 in one shot
        glchk_stmt!(
            gl::ProgramUniform3fv(
                self.program,
                self.uniform_locations[VAR_IDX_ORIGIN],
                1,
                origin.data.as_ptr(),
            );

            gl::ProgramUniform3fv(
                self.program,
                self.uniform_locations[VAR_IDX_SCREEN_DOT_TOP_LEFT],
                1,
                top_left.data.as_ptr(),
            );

            gl::ProgramUniform3fv(
                self.program,
                self.uniform_locations[VAR_IDX_SCREEN_DOT_LEFT],
                1,
                left.data.as_ptr(),
            );

            gl::ProgramUniform3fv(
                self.program,
                self.uniform_locations[VAR_IDX_SCREEN_DOT_UP],
                1,
                up.data.as_ptr(),
            );
        );
        // FIXME-END

        Ok(())
    }
}
