use ash::vk;

use crate::context::Context;
use crate::datatypes::*;

use std::sync::Arc;

#[derive(PartialOrd, Ord, PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub enum BlasName {
    Chunk(i32, i32),
    Dog,
}

#[derive(Copy, Clone)]
pub struct GeometryInstance {
    pub transform: [f32; 12],
    pub instance_custom_index: u32,
    pub mask: u32,
    pub instance_offset: u32,
    pub flags: vk::GeometryInstanceFlagsNV,
    pub acceleration_structure_handle: u64,
}

impl GeometryInstance {
    pub fn get_data(&self) -> GeometryInstanceData {
        let instance_custom_indexand_mask =
            (self.mask << 24) | (self.instance_custom_index & 0x00_ff_ff_ff);
        let instance_offset_and_flags =
            (self.flags.as_raw() << 24) | (self.instance_offset & 0x00_ff_ff_ff);
        GeometryInstanceData {
            transform: self.transform,
            instance_custom_indexand_mask,
            instance_offset_and_flags,
            acceleration_structure_handle: self.acceleration_structure_handle,
        }
    }
}

#[derive(Copy, Clone)]
#[allow(dead_code)]
#[repr(C)]
pub struct GeometryInstanceData {
    transform: [f32; 12],
    instance_custom_indexand_mask: u32,
    instance_offset_and_flags: u32,
    acceleration_structure_handle: u64,
}

#[allow(dead_code)]
pub struct BlasVariable {
    pub is_build: bool,
    acceleration_structure: AccelerationStructure,
    instance_data: GeometryInstanceData,

    triangle_data: BufferVariable,
    textures: BufferVariable,
    column_colors: BufferVariable,

    // those fields are unused but the :evice-memory they handle need to be freed at the same time than the whole structure
    geometries: Vec<vk::GeometryNV>,
    vertices: BufferVariable,
    indices: BufferVariable,
}

impl BlasVariable {
    pub fn from_geometry(
        context: &Arc<Context>,
        vertices: BufferVariable,
        indices: BufferVariable,
        triangle_data: BufferVariable,
        textures: BufferVariable,
        column_colors: BufferVariable,
        vertex_stride: usize,
    ) -> BlasVariable {
        ///// Create geometries list
        let geometries = vec![vk::GeometryNV::builder()
            .geometry_type(vk::GeometryTypeNV::TRIANGLES)
            .geometry(
                vk::GeometryDataNV::builder()
                    .triangles(
                        vk::GeometryTrianglesNV::builder()
                            .vertex_data(*vertices.buffer())
                            .vertex_count(vertices.element_count() as _)
                            .vertex_offset(0)
                            .vertex_stride(vertex_stride as _)
                            .vertex_format(vk::Format::R32G32B32A32_SFLOAT)
                            .index_data(*indices.buffer())
                            .index_count(indices.element_count() as _)
                            .index_offset(0)
                            .index_type(vk::IndexType::UINT32)
                            .transform_data(vk::Buffer::null())
                            .transform_offset(0)
                            .build(),
                    )
                    .build(),
            )
            .flags(vk::GeometryFlagsNV::NO_DUPLICATE_ANY_HIT_INVOCATION)
            .build()];

        let acceleration_structure_info = vk::AccelerationStructureInfoNV::builder()
            .ty(vk::AccelerationStructureTypeNV::BOTTOM_LEVEL)
            .geometries(&geometries)
            .build();

        let acceleration_structure =
            AccelerationStructure::new(Arc::clone(context), acceleration_structure_info);

        ///// Create instance
        let transform = [1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0];

        let geometry_instance = GeometryInstance {
            transform,
            instance_custom_index: 0,
            mask: 0xff,
            instance_offset: 0,
            flags: vk::GeometryInstanceFlagsNV::TRIANGLE_CULL_DISABLE_NV,
            acceleration_structure_handle: acceleration_structure.handle,
        };
        let instance_data = geometry_instance.get_data();

        BlasVariable {
            acceleration_structure,
            instance_data,
            triangle_data,
            textures,
            is_build: false,
            geometries,
            vertices,
            indices,
            column_colors,
        }
    }

    pub fn triangle_data(&self) -> &BufferVariable {
        &self.triangle_data
    }

    pub fn textures(&self) -> &BufferVariable {
        &self.textures
    }

    pub fn column_colors(&self) -> &BufferVariable {
        &self.column_colors
    }

    pub fn bindings(&self) -> InstanceBinding {
        InstanceBinding {
            indices: vk::Buffer::null(),
            triangles: vk::Buffer::null(),
        }
    }

    pub fn instance_data(&self) -> GeometryInstanceData {
        self.instance_data
    }

    pub fn build_memory_requirements(&self) -> Option<u64> {
        match self.is_build {
            true => None,
            false => Some(
                self.acceleration_structure
                    .get_memory_requirements(
                        vk::AccelerationStructureMemoryRequirementsTypeNV::BUILD_SCRATCH,
                    )
                    .memory_requirements
                    .size,
            ),
        }
    }

    pub fn build(
        &mut self,
        command_buffer: vk::CommandBuffer,
        scratch_buffer: &BufferVariable,
    ) -> bool {
        if !self.is_build {
            self.is_build = true;
            self.acceleration_structure
                .cmd_build(command_buffer, scratch_buffer, None);

            true
        } else {
            false
        }
    }
}
