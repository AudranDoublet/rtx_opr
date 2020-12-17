use crate::context::{find_memory_type, Context};
use ash::version::DeviceV1_0;
use ash::vk;

use std::sync::Arc;

use crate::datatypes::BufferVariable;

pub struct AccelerationStructure {
    pub context: Arc<Context>,
    pub handle: u64,
    pub acceleration_structure_info: vk::AccelerationStructureInfoNV,
    pub acceleration_structure: vk::AccelerationStructureNV,
    pub memory: vk::DeviceMemory,
}

impl AccelerationStructure {
    pub fn create_top(context: Arc<Context>, instance_count: u32) -> Self {
        let acceleration_structure_info = vk::AccelerationStructureInfoNV::builder()
            .ty(vk::AccelerationStructureTypeNV::TOP_LEVEL)
            .instance_count(instance_count)
            .build();
        Self::new(context, acceleration_structure_info)
    }

    pub fn new(
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
