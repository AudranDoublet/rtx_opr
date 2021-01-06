mod builder;

pub use builder::*;

use ash::version::DeviceV1_0;
use ash::vk;
use std::sync::Arc;

use crate::context::Context;

pub struct DescriptorSet {
    context: Arc<Context>,
    pub layout: vk::DescriptorSetLayout,
    pub pool : vk::DescriptorPool,
    pub set : vk::DescriptorSet,
}


impl Drop for DescriptorSet {
    fn drop(&mut self) {
        let device = self.context.device();
        unsafe {
            device.destroy_descriptor_pool(self.pool, None);
            device.destroy_descriptor_set_layout(self.layout, None);
        }
    }
}
