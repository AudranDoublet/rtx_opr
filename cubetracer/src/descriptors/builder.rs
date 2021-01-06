// use ash::extensions::nv::RayTracing;
use ash::version::DeviceV1_0;
use std::collections::HashMap;
use std::sync::Arc;

use ash::vk;

use crate::pipeline::ShaderType;
use crate::context::Context;
use crate::datatypes::*;
use crate::descriptors::DescriptorSet;

pub struct DescriptorSetBuilder<'a> {
    context: Arc<Context>,

    descriptor_counts: HashMap<vk::DescriptorType, u32>,
    bindings: Vec<vk::DescriptorSetLayoutBinding>,
    variables: Vec<(vk::DescriptorType, &'a mut dyn DataType)>,
}


impl<'a> DescriptorSetBuilder<'a> {
    pub fn new(context: &Arc<Context>) -> Self {
        Self {
            context: Arc::clone(context),

            descriptor_counts: HashMap::new(),
            bindings: Vec::new(),
            variables: Vec::new(),
        }
    }

    pub fn binding(
        &mut self,
        desc_type: vk::DescriptorType,
        variable: &'a mut dyn DataType,
        stages: &[ShaderType],
    ) -> &mut Self {
        self.binding_count(desc_type, 1, variable, stages)
    }

    pub fn bindings(
        &mut self,
        desc_type: vk::DescriptorType,
        variables: Vec<&'a mut dyn DataType>,
        stages: &[ShaderType],
    ) -> &mut Self {
        for v in variables {
            self.binding(desc_type, v, stages);
        }

        self
    }

    pub fn binding_count(
        &mut self,
        desc_type: vk::DescriptorType,
        descriptor_count: u32,
        variable: &'a mut dyn DataType,
        stages: &[ShaderType],
    ) -> &mut Self {
        let stage_flags = stages
            .iter()
            .map(|s| s.stage())
            .fold(vk::ShaderStageFlags::empty(), |a, b| a | b);

        self.bindings.push(
            vk::DescriptorSetLayoutBinding::builder()
                .binding(self.bindings.len() as u32)
                .descriptor_type(desc_type)
                .descriptor_count(descriptor_count)
                .stage_flags(stage_flags)
                .build(),
        );

        *self.descriptor_counts.entry(desc_type).or_insert(0) += descriptor_count;
        self.variables.push((desc_type, variable));

        self
    }

    pub fn build(&mut self) -> DescriptorSet {
        let descriptor_set_layout = unsafe {
            self.context
                .device()
                .create_descriptor_set_layout(
                    &vk::DescriptorSetLayoutCreateInfo::builder().bindings(&self.bindings),
                    None,
                )
                .expect("Failed to create descriptor set layout")
        };

        /////////// CREATE DESCRIPTORS
        let descriptor_pool = self.create_descriptor_pool();
        let device = self.context.device();

        let descriptor_set = {
            let set_layouts = [descriptor_set_layout];
            let allocate_info = vk::DescriptorSetAllocateInfo::builder()
                .descriptor_pool(descriptor_pool)
                .set_layouts(&set_layouts);
            let sets = unsafe {
                device
                    .allocate_descriptor_sets(&allocate_info)
                    .expect("Failed to allocate descriptor set")
            };

            let write_descriptor_sets = self
                .variables
                .iter_mut()
                .enumerate()
                .map(|(i, (desc_type, var))| {
                    let mut info = var
                        .write_descriptor_builder()
                        .dst_set(sets[0])
                        .dst_binding(i as u32)
                        .descriptor_type(*desc_type)
                        .build();

                    if *desc_type == vk::DescriptorType::ACCELERATION_STRUCTURE_NV {
                        info.descriptor_count = 1;
                    }

                    info
                })
                .collect::<Vec<_>>();

            unsafe { device.update_descriptor_sets(&write_descriptor_sets, &[]) };

            sets[0]
        };

        DescriptorSet {
            context: Arc::clone(&self.context),
            layout: descriptor_set_layout,
            pool: descriptor_pool,
            set: descriptor_set,
        }
    }

}

impl<'a> DescriptorSetBuilder<'a> {
    fn create_descriptor_pool(&self) -> vk::DescriptorPool {
        let pool_sizes = self
            .descriptor_counts
            .iter()
            .map(|(desc, count)| {
                vk::DescriptorPoolSize::builder()
                    .ty(*desc)
                    .descriptor_count(*count)
                    .build()
            })
            .collect::<Vec<_>>();

        let pool_create_info = vk::DescriptorPoolCreateInfo::builder()
            .max_sets(1)
            .pool_sizes(&pool_sizes);

        unsafe {
            self.context
                .device()
                .create_descriptor_pool(&pool_create_info, None)
                .expect("Failed to create descriptor pool")
        }
    }
}
