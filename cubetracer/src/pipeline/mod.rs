mod builder;
mod update_factory;
mod shader;

pub use builder::*;
pub use shader::*;
pub use update_factory::*;

use std::sync::Arc;


use ash::version::DeviceV1_0;
use ash::vk;

use crate::context::Context;
use crate::datatypes::BufferVariable;


pub trait Pipeline {
    fn pipeline(&self) -> vk::Pipeline;
    fn pipeline_layout(&self) -> vk::PipelineLayout;
    fn bind_point(&self) -> vk::PipelineBindPoint;
    fn descriptor_sets(&self) -> &Vec<vk::DescriptorSet>;

    fn bind(&self, context: &Arc<Context>, buffer: vk::CommandBuffer) {
        let bind_point = self.bind_point();

        // Bind pipeline
        unsafe {
            context.device().cmd_bind_pipeline(
                buffer,
                bind_point,
                self.pipeline(),
            )
        };

        // Bind descriptor set
        unsafe {
            context.device().cmd_bind_descriptor_sets(
                buffer,
                bind_point,
                self.pipeline_layout(),
                0,
                self.descriptor_sets(),
                &[],
            );
        };
    }
}

pub struct RaytracerPipeline {
    pub rt_properties: vk::PhysicalDeviceRayTracingPropertiesNV,
    pub pipeline: vk::Pipeline,
    pub pipeline_layout: vk::PipelineLayout,
    pub shader_binding_table_buffer: BufferVariable,
    pub descriptor_sets: Vec<vk::DescriptorSet>,
    pub context: Arc<Context>,
    pub miss_offset: usize,
    pub hit_offset: usize,
}

impl RaytracerPipeline {
    pub fn dispatch(&self, buffer: vk::CommandBuffer, width: u32, height: u32, id_raygen: u32) {
        // Trace rays
        let shader_group_handle_size = self.rt_properties.shader_group_handle_size;
        let miss_offset = self.miss_offset as u32 * shader_group_handle_size;
        let hit_offset = self.hit_offset as u32 * shader_group_handle_size;

        unsafe {
            let sbt_buffer = *self.shader_binding_table_buffer.buffer();

            // initial rays
            self.context.ray_tracing().cmd_trace_rays(
                buffer,
                sbt_buffer,
                (id_raygen * shader_group_handle_size).into(), // raygen offset
                sbt_buffer,
                miss_offset.into(),
                shader_group_handle_size.into(),
                sbt_buffer,
                hit_offset.into(),
                shader_group_handle_size.into(),
                vk::Buffer::null(),
                0,
                0,
                width,
                height,
                1,
            );

        }
    }
}

impl Pipeline for RaytracerPipeline {
    fn bind_point(&self) -> vk::PipelineBindPoint {
        vk::PipelineBindPoint::RAY_TRACING_NV
    }

    fn pipeline(&self) -> vk::Pipeline {
        self.pipeline
    }

    fn pipeline_layout(&self) -> vk::PipelineLayout {
        self.pipeline_layout
    }

    fn descriptor_sets(&self) -> &Vec<vk::DescriptorSet> {
        &self.descriptor_sets
    }
}


impl Drop for RaytracerPipeline {
    fn drop(&mut self) {
        let device = self.context.device();
        unsafe {
            device.destroy_pipeline(self.pipeline, None);
            device.destroy_pipeline_layout(self.pipeline_layout, None);
        }
    }
}
