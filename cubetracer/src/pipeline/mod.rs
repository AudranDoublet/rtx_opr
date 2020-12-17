mod builder;
mod shader;

pub use builder::*;
pub use shader::*;

use std::sync::Arc;

use crate::context::Context;
use crate::datatypes::DataType;
use crate::datatypes::BufferVariable;

use ash::vk;
use ash::version::DeviceV1_0;

pub struct RaytracerPipeline {
    pub rt_properties: vk::PhysicalDeviceRayTracingPropertiesNV,
    pub pipeline: vk::Pipeline,
    pub pipeline_layout: vk::PipelineLayout,
    pub shader_binding_table_buffer: BufferVariable,
    pub descriptor_set_layout: vk::DescriptorSetLayout,
    pub descriptor_pool: vk::DescriptorPool,
    pub descriptor_sets: Vec<vk::DescriptorSet>,
    pub desc_types: Vec<vk::DescriptorType>,
    pub context: Arc<Context>,
}

impl RaytracerPipeline {
    pub fn update_binding(
        &mut self,
        binding_id: usize,
        variable: &mut dyn DataType
    ) -> &mut Self {
        let desc_type = self.desc_types[binding_id];

        let write_descriptor_sets = vec![{
            let mut info = variable
                .write_descriptor_builder()
                .dst_set(self.descriptor_sets[0])
                .dst_binding(binding_id as u32)
                .descriptor_type(desc_type)
                .build();

            if desc_type == vk::DescriptorType::ACCELERATION_STRUCTURE_NV {
                info.descriptor_count = 1;
            }

            info
        }];

        unsafe {
            self.context.device().update_descriptor_sets(&write_descriptor_sets, &[]);
        }

        self
    }
}
