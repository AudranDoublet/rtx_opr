mod raytracing;
mod update_factory;
mod shader;
mod compute;

pub use raytracing::*;
pub use compute::*;
pub use shader::*;
pub use update_factory::*;

use std::sync::Arc;


use ash::version::DeviceV1_0;
use ash::vk;

use crate::context::Context;

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
