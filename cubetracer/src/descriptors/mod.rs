mod builder;

pub use builder::*;

use ash::version::DeviceV1_0;
use ash::vk;
use std::sync::Arc;

use crate::context::Context;
use crate::datatypes::DataType;

pub struct DescriptorSet {
    context: Arc<Context>,
    pub layout: vk::DescriptorSetLayout,
    pub pool: vk::DescriptorPool,
    pub set: vk::DescriptorSet,
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

impl DescriptorSet {
    pub fn update<'a>(&'a self, context: &Arc<Context>) -> DescriptorUpdateFactory<'a> {
        DescriptorUpdateFactory::new(self, context)
    }
}

pub struct DescriptorUpdateFactory<'a> {
    context: Arc<Context>,
    descriptor_set: &'a DescriptorSet,
    variables: Vec<(u32, vk::DescriptorType, &'a mut dyn DataType)>,
}

impl<'a> DescriptorUpdateFactory<'a> {
    pub fn new(descriptor_set: &'a DescriptorSet, context: &Arc<Context>) -> Self {
        Self {
            descriptor_set,
            context: Arc::clone(context),
            variables: Vec::new(),
        }
    }

    /// add a new memory binding for shaders
    pub fn register(
        &mut self,
        idx: u32,
        desc_type: vk::DescriptorType,
        variable: &'a mut dyn DataType,
    ) -> &mut Self {
        self.variables.push((idx, desc_type, variable));
        self
    }

    pub fn update(&mut self) {
        let device = self.context.device();
        let set = self.descriptor_set.set;

        let write_descriptor_sets = self
            .variables
            .iter_mut()
            .map(|(idx, desc_type, var)| {
                let mut info = var
                    .write_descriptor_builder()
                    .dst_set(set)
                    .dst_binding(*idx as u32)
                    .descriptor_type(*desc_type)
                    .build();

                info.descriptor_count = var.len() as u32;
                info
            })
            .collect::<Vec<_>>();

        unsafe { device.update_descriptor_sets(&write_descriptor_sets, &[]) };
    }
}
