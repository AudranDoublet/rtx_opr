use ash::vk;

use crate::context::Context;
use crate::datatypes::*;
use crate::mesh::Vertex;

use std::sync::Arc;
use std::mem::size_of;

#[derive(Copy, Clone)]
#[allow(dead_code)]
#[repr(C)]
pub struct GeometryInstanceData {
    transform: [f32; 12],
    instance_custom_indexand_mask: u32,
    instance_offset_and_flags: u32,
    acceleration_structure_handle: u64,
}

pub struct BlasVariable {
    pub acceleration_structure: AccelerationStructure,
    pub instance_buffer: BufferVariable,
    pub geometries: Vec<vk::GeometryNV>,
}

impl BlasVariable {
    pub fn from_geometry(
        context: &Arc<Context>,
        vertices: &BufferVariable,
        indices: &BufferVariable
    ) -> BlasVariable {
        ///// Create geometries list
        let geometries = vec![
            vk::GeometryNV::builder()
            .geometry_type(vk::GeometryTypeNV::TRIANGLES)
            .geometry(
                vk::GeometryDataNV::builder()
                    .triangles(
                        vk::GeometryTrianglesNV::builder()
                            .vertex_data(*vertices.buffer())
                            .vertex_count(vertices.element_count() as _)
                            .vertex_offset(0)
                            .vertex_stride(size_of::<Vertex>() as _)
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
            .flags(vk::GeometryFlagsNV::OPAQUE)
            .build()
        ];

        let acceleration_structure_info = vk::AccelerationStructureInfoNV::builder()
            .ty(vk::AccelerationStructureTypeNV::BOTTOM_LEVEL)
            .geometries(&geometries)
            .build();

        let acceleration_structure = AccelerationStructure::new(
            Arc::clone(context), acceleration_structure_info
        );

        ///// Create instance
        let transform = [
            1.0, 0.0, 0.0, 0.0,
            0.0, 1.0, 0.0, 0.0,
            0.0, 0.0, 1.0, 0.0,
        ];

        let instance_buffer = {
            let geometry_instance = GeometryInstance {
                transform,
                instance_custom_index: 0,
                mask: 0xff,
                instance_offset: 0,
                flags: vk::GeometryInstanceFlagsNV::TRIANGLE_CULL_DISABLE_NV,
                acceleration_structure_handle: acceleration_structure.handle,
            };
            let geometry_instance = geometry_instance.get_data();
            unsafe {
                BufferVariable::device_buffer(
                    context,
                    vk::BufferUsageFlags::RAY_TRACING_NV,
                    any_as_u8_slice(&geometry_instance),
                )
                .0
            }
        };

        BlasVariable {
            acceleration_structure,
            instance_buffer,
            geometries,
        }
    }

    pub fn instance(&self) -> &BufferVariable {
        &self.instance_buffer
    }
}

unsafe fn any_as_u8_slice<T: Sized>(any: &T) -> &[u8] {
    let ptr = (any as *const T) as *const u8;
    std::slice::from_raw_parts(ptr, std::mem::size_of::<T>())
}
