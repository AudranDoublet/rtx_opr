use crate::context::{find_memory_type, Context};
use ash::version::DeviceV1_0;
use ash::vk;

use std::mem::size_of;
use std::sync::Arc;

use crate::datatypes::{BufferVariable, DataType};
use crate::mesh::Vertex;

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

pub struct AccelerationStructure {
    pub context: Arc<Context>,
    pub handle: u64,
    pub acceleration_structure_info: vk::AccelerationStructureInfoNV,
    pub acceleration_structure: vk::AccelerationStructureNV,
    pub memory: vk::DeviceMemory,
}

impl AccelerationStructure {
    pub fn create_bottom(context: Arc<Context>, geometries: &[vk::GeometryNV]) -> Self {
        let acceleration_structure_info = vk::AccelerationStructureInfoNV::builder()
            .ty(vk::AccelerationStructureTypeNV::BOTTOM_LEVEL)
            .geometries(geometries)
            .build();
        Self::new(context, acceleration_structure_info)
    }

    pub fn create_top(context: Arc<Context>, instance_count: u32) -> Self {
        let acceleration_structure_info = vk::AccelerationStructureInfoNV::builder()
            .ty(vk::AccelerationStructureTypeNV::TOP_LEVEL)
            .instance_count(instance_count)
            .build();
        Self::new(context, acceleration_structure_info)
    }

    fn new(
        context: Arc<Context>,
        acceleration_structure_info: vk::AccelerationStructureInfoNV,
    ) -> Self {
        let ray_tracing = context.ray_tracing();

        let acceleration_structure = {
            let acceleration_structure_create_info =
                vk::AccelerationStructureCreateInfoNV::builder().info(acceleration_structure_info);

            unsafe {
                ray_tracing
                    .create_acceleration_structure(&acceleration_structure_create_info, None)
                    .expect("Failed to create acceleration structure")
            }
        };

        let memory = {
            let mem_requirements_info =
                vk::AccelerationStructureMemoryRequirementsInfoNV::builder()
                    .ty(vk::AccelerationStructureMemoryRequirementsTypeNV::OBJECT)
                    .acceleration_structure(acceleration_structure);
            let mem_requirements = unsafe {
                ray_tracing.get_acceleration_structure_memory_requirements(&mem_requirements_info)
            };

            let memory_allocate_info = vk::MemoryAllocateInfo::builder()
                .allocation_size(mem_requirements.memory_requirements.size)
                .memory_type_index(find_memory_type(
                    mem_requirements.memory_requirements,
                    context.get_mem_properties(),
                    vk::MemoryPropertyFlags::DEVICE_LOCAL,
                ));
            unsafe {
                context
                    .device()
                    .allocate_memory(&memory_allocate_info, None)
                    .expect("Failed to allocate memory")
            }
        };

        {
            let acceleration_structure_memory_info =
                [vk::BindAccelerationStructureMemoryInfoNV::builder()
                    .acceleration_structure(acceleration_structure)
                    .memory(memory)
                    .build()];

            unsafe {
                ray_tracing
                    .bind_acceleration_structure_memory(&acceleration_structure_memory_info)
                    .expect("Failed to bind acceleration structure")
            };
        }

        let handle = unsafe {
            ray_tracing
                .get_acceleration_structure_handle(acceleration_structure)
                .expect("Failed to get acceleration structure handle")
        };
        Self {
            context,
            acceleration_structure_info,
            acceleration_structure,
            memory,
            handle,
        }
    }
}

impl AccelerationStructure {
    pub fn get_memory_requirements(
        &self,
        requirements_type: vk::AccelerationStructureMemoryRequirementsTypeNV,
    ) -> vk::MemoryRequirements2 {
        let mem_requirements_info = vk::AccelerationStructureMemoryRequirementsInfoNV::builder()
            .ty(requirements_type)
            .acceleration_structure(self.acceleration_structure);
        unsafe {
            self.context
                .ray_tracing()
                .get_acceleration_structure_memory_requirements(&mem_requirements_info)
        }
    }

    pub fn cmd_build(
        &self,
        command_buffer: vk::CommandBuffer,
        scratch_buffer: &BufferVariable,
        instance_buffer: Option<&BufferVariable>,
    ) {
        let instance_buffer = instance_buffer.map_or(vk::Buffer::null(), |buffer| *buffer.buffer());
        unsafe {
            self.context.ray_tracing().cmd_build_acceleration_structure(
                command_buffer,
                &self.acceleration_structure_info,
                instance_buffer,
                0,
                false,
                self.acceleration_structure,
                vk::AccelerationStructureNV::null(),
                *scratch_buffer.buffer(),
                0,
            )
        };
    }
}

impl Drop for AccelerationStructure {
    fn drop(&mut self) {
        let ray_tracing = self.context.ray_tracing();
        let device = self.context.device();
        unsafe {
            device.free_memory(self.memory, None);
            ray_tracing.destroy_acceleration_structure(self.acceleration_structure, None);
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

pub struct ASVariable {
    pub top_as: AccelerationStructure,
    pub bot_as: AccelerationStructure,
    structures: Vec<vk::AccelerationStructureNV>,
    info: Vec<vk::WriteDescriptorSetAccelerationStructureNV>,
}

impl ASVariable {
    pub fn new(
        context: &Arc<Context>,
        vertices: &BufferVariable,
        indices: &BufferVariable,
    ) -> ASVariable {
        let geometries = [vk::GeometryNV::builder()
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
            .build()];

        let bottom_as = AccelerationStructure::create_bottom(Arc::clone(context), &geometries);

        // Top acceleration structure
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
                flags: vk::GeometryInstanceFlagsNV::TRIANGLE_CULL_DISABLE,
                acceleration_structure_handle: bottom_as.handle,
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
        let top_as = AccelerationStructure::create_top(Arc::clone(context), 1);

        // Build acceleration structure
        let bottom_mem_requirements = bottom_as.get_memory_requirements(
            vk::AccelerationStructureMemoryRequirementsTypeNV::BUILD_SCRATCH,
        );
        let top_mem_requirements = top_as.get_memory_requirements(
            vk::AccelerationStructureMemoryRequirementsTypeNV::BUILD_SCRATCH,
        );

        let scratch_buffer_size = bottom_mem_requirements
            .memory_requirements
            .size
            .max(top_mem_requirements.memory_requirements.size);
        let scratch_buffer = BufferVariable::create(
            context,
            scratch_buffer_size,
            scratch_buffer_size as _,
            vk::BufferUsageFlags::RAY_TRACING_NV,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
        );

        context.execute_one_time_commands(|command_buffer| {
            // Build bottom AS
            bottom_as.cmd_build(command_buffer, &scratch_buffer, None);

            let memory_barrier = [vk::MemoryBarrier::builder()
                .src_access_mask(
                    vk::AccessFlags::ACCELERATION_STRUCTURE_READ_NV
                        | vk::AccessFlags::ACCELERATION_STRUCTURE_WRITE_NV,
                )
                .dst_access_mask(
                    vk::AccessFlags::ACCELERATION_STRUCTURE_READ_NV
                        | vk::AccessFlags::ACCELERATION_STRUCTURE_WRITE_NV,
                )
                .build()];
            unsafe {
                context.device().cmd_pipeline_barrier(
                    command_buffer,
                    vk::PipelineStageFlags::ACCELERATION_STRUCTURE_BUILD_NV,
                    vk::PipelineStageFlags::ACCELERATION_STRUCTURE_BUILD_NV,
                    vk::DependencyFlags::empty(),
                    &memory_barrier,
                    &[],
                    &[],
                )
            };

            // Build top AS
            top_as.cmd_build(command_buffer, &scratch_buffer, Some(&instance_buffer));

            unsafe {
                context.device().cmd_pipeline_barrier(
                    command_buffer,
                    vk::PipelineStageFlags::ACCELERATION_STRUCTURE_BUILD_NV,
                    vk::PipelineStageFlags::ACCELERATION_STRUCTURE_BUILD_NV,
                    vk::DependencyFlags::empty(),
                    &memory_barrier,
                    &[],
                    &[],
                )
            };
        });

        ASVariable {
            top_as,
            bot_as: bottom_as,
            structures: Vec::new(),
            info: Vec::new(),
        }
    }
}

impl DataType for ASVariable {
    fn write_descriptor_builder(&mut self) -> vk::WriteDescriptorSetBuilder {
        self.structures.push(self.top_as.acceleration_structure);
        self.info.push(
            vk::WriteDescriptorSetAccelerationStructureNV::builder()
                .acceleration_structures(&self.structures)
                .build(),
        );

        vk::WriteDescriptorSet::builder().push_next(&mut self.info[0])
    }
}

unsafe fn any_as_u8_slice<T: Sized>(any: &T) -> &[u8] {
    let ptr = (any as *const T) as *const u8;
    std::slice::from_raw_parts(ptr, std::mem::size_of::<T>())
}
