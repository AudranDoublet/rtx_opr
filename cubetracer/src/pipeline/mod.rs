mod builder;
mod shader;

pub use builder::*;
pub use shader::*;

use crate::datatypes::BufferVariable;
use ash::vk;

pub struct RaytracerPipeline {
    pub rt_properties: vk::PhysicalDeviceRayTracingPropertiesNV,
    pub pipeline: vk::Pipeline,
    pub pipeline_layout: vk::PipelineLayout,
    pub shader_binding_table_buffer: BufferVariable,
    pub descriptor_set_layout: vk::DescriptorSetLayout,
    pub descriptor_pool: vk::DescriptorPool,
    pub descriptor_sets: Vec<vk::DescriptorSet>,
}
