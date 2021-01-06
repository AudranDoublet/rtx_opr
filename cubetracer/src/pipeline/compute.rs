use ash::version::DeviceV1_0;
use ash::vk;

use crate::context::Context;
use std::ffi::CString;
use std::sync::Arc;

use crate::pipeline::*;
use crate::descriptors::*;

pub struct ComputePipelineBuilder {
    folder: String,

    context: Arc<Context>,
    entry_point: CString,

    descriptor_sets: Vec<vk::DescriptorSet>,
    descriptor_set_layouts: Vec<vk::DescriptorSetLayout>,

    shaders: Vec<(ShaderType, ShaderModule)>,
}

impl ComputePipelineBuilder {
    pub fn new(context: &Arc<Context>, folder: &str) -> Self {
        Self {
            context: Arc::clone(context),
            folder: folder.to_string(),

            shaders: Vec::new(),

            descriptor_set_layouts: Vec::new(),
            descriptor_sets: Vec::new(),

            entry_point: CString::new("main").unwrap(),
        }
    }

    /// add a descriptor set to pipeline
    pub fn descriptor_set(&mut self, descriptor_set: &DescriptorSet) -> &mut Self {
        self.descriptor_sets.push(descriptor_set.set);
        self.descriptor_set_layouts.push(descriptor_set.layout);
        self
    }

    /// retrieve created pipeline stages
    fn stages(&self) -> Vec<vk::PipelineShaderStageCreateInfo> {
        let mut stages = Vec::new();

        for (shader_type, shader) in &self.shaders {
            stages.push(
                vk::PipelineShaderStageCreateInfo::builder()
                    .stage(shader_type.stage())
                    .module(shader.module())
                    .name(&self.entry_point)
                    .build(),
            );
        }

        stages
    }

    fn load_shader(&mut self, shader_type: ShaderType, name: &str) -> u32 {
        let shader = ShaderModule::new(
            Arc::clone(&self.context),
            format!("{}/{}", self.folder, name),
        );

        self.shaders.push((shader_type, shader));
        self.shaders.len() as u32 - 1
    }

    /// load and crate a shader
    pub fn shader(&mut self, name: &str) -> &mut Self {
        if self.shaders.len() > 0 {
            panic!("compute shader pipeline supports only one shader!");
        }

        let _shader_id = self.load_shader(ShaderType::Compute, name);
        self
    }

    pub fn build(&mut self) -> ComputePipeline {
        /////////// CREATE PIPELINE LAYOUT
        let pipeline_layout_create_info =
            vk::PipelineLayoutCreateInfo::builder().set_layouts(&self.descriptor_set_layouts);
        let pipeline_layout = unsafe {
            self.context
                .device()
                .create_pipeline_layout(&pipeline_layout_create_info, None)
                .expect("Failed to create pipeline layout")
        };

        /////////// CREATE PIPELINE
        let stages = self.stages();

        let pipeline_create_info = [
            vk::ComputePipelineCreateInfo::builder()
            .stage(stages[0])
            .layout(pipeline_layout)
            .build()
        ];

        let pipeline = unsafe {
            self.context
                .device()
                .create_compute_pipelines(
                    vk::PipelineCache::null(),
                    &pipeline_create_info,
                    None,
                )
                .expect("Failed to create compute pipeline")[0]
        };

        ComputePipeline {
            pipeline,
            pipeline_layout,
            descriptor_sets: self.descriptor_sets.clone(),
            context: Arc::clone(&self.context),
        }
    }
}

pub struct ComputePipeline {
    pub pipeline: vk::Pipeline,
    pub pipeline_layout: vk::PipelineLayout,
    pub descriptor_sets: Vec<vk::DescriptorSet>,
    pub context: Arc<Context>,
}

impl ComputePipeline {
    pub fn dispatch(&self, buffer: vk::CommandBuffer, width: u32, height: u32) {
        unsafe {
            self.context.device().cmd_dispatch(
                buffer,
                width,
                height,
                1,
            );
        }
    }
}

impl Pipeline for ComputePipeline {
    fn bind_point(&self) -> vk::PipelineBindPoint {
        vk::PipelineBindPoint::COMPUTE
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


impl Drop for ComputePipeline {
    fn drop(&mut self) {
        let device = self.context.device();
        unsafe {
            device.destroy_pipeline(self.pipeline, None);
            device.destroy_pipeline_layout(self.pipeline_layout, None);
        }
    }
}
