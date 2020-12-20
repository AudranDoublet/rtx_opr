use ash::vk;
use ash::version::DeviceV1_0;

use crate::context::Context;
use std::sync::Arc;

use crate::datatypes::*;
use crate::pipeline::*;


pub struct UpdateFactory<'a> {
    context: Arc<Context>,
    variables: Vec<(u32, vk::DescriptorType, &'a mut dyn DataType)>,
}

impl<'a> UpdateFactory<'a> {
    pub fn new(context: &Arc<Context>) -> UpdateFactory<'a> {
        UpdateFactory {
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

    pub fn update(&mut self, pipeline: &mut RaytracerPipeline) {
        let device = self.context.device();

        let write_descriptor_sets = self
            .variables
            .iter_mut()
            .map(|(idx, desc_type, var)| {
                let mut info = var
                    .write_descriptor_builder()
                    .dst_set(pipeline.descriptor_sets[0])
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
