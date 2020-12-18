use ash::extensions::nv::RayTracing;
use ash::version::DeviceV1_0;
use ash::vk;

use crate::context::Context;
use std::ffi::CString;
use std::sync::Arc;

use crate::datatypes::*;
use crate::pipeline::*;

use std::collections::HashMap;

trait BuilderWithType {
    fn shader_with_type(self, i: u32, s_type: ShaderType) -> Self;
}

impl<'a> BuilderWithType for vk::RayTracingShaderGroupCreateInfoNVBuilder<'a> {
    fn shader_with_type(self, i: u32, s_type: ShaderType) -> Self {
        match s_type {
            ShaderType::ClosestHit => self
                .closest_hit_shader(i)
                .general_shader(vk::SHADER_UNUSED_NV)
                .any_hit_shader(vk::SHADER_UNUSED_NV)
                .intersection_shader(vk::SHADER_UNUSED_NV),
            _ => self
                .general_shader(i)
                .closest_hit_shader(vk::SHADER_UNUSED_NV)
                .any_hit_shader(vk::SHADER_UNUSED_NV)
                .intersection_shader(vk::SHADER_UNUSED_NV),
        }
    }
}

pub struct PipelineBuilder<'a> {
    shader_group_id: u32,
    binding_id: u32,
    folder: String,

    context: Arc<Context>,
    entry_point: CString,

    variables: Vec<(vk::DescriptorType, &'a mut dyn DataType)>,
    descriptor_counts: HashMap<vk::DescriptorType, u32>,
    bindings: Vec<vk::DescriptorSetLayoutBinding>,
    shaders: Vec<(u32, ShaderType, Option<ShaderModule>)>,
}

impl<'a> PipelineBuilder<'a> {
    pub fn new(context: &Arc<Context>, folder: &str) -> PipelineBuilder<'a> {
        PipelineBuilder {
            shader_group_id: 0,
            binding_id: 0,

            context: Arc::clone(context),
            folder: folder.to_string(),

            descriptor_counts: HashMap::new(),
            bindings: Vec::new(),
            shaders: Vec::new(),
            variables: Vec::new(),

            entry_point: CString::new("main").unwrap(),
        }
    }

    /// retrieve created pipeline stages
    fn stages(&self) -> Vec<vk::PipelineShaderStageCreateInfo> {
        let mut stages = Vec::new();

        for (_, shader_type, shader) in &self.shaders {
            if let Some(shader) = shader {
                stages.push(
                    vk::PipelineShaderStageCreateInfo::builder()
                        .stage(shader_type.stage())
                        .module(shader.module())
                        .name(&self.entry_point)
                        .build(),
                );
            }
        }

        stages
    }

    /// retrieve created raytracing groups
    fn groups(&self) -> Vec<vk::RayTracingShaderGroupCreateInfoNV> {
        let mut groups = Vec::new();

        for (shader_group_id, shader_type, _) in &self.shaders {
            groups.push(
                vk::RayTracingShaderGroupCreateInfoNV::builder()
                    .ty(shader_type.group())
                    .shader_with_type(*shader_group_id, *shader_type)
                    .build(),
            );
        }

        groups
    }

    /// add a new memory binding for shaders
    pub fn binding(
        &mut self,
        desc_type: vk::DescriptorType,
        variable: &'a mut dyn DataType,
        stages: &[ShaderType],
    ) -> &mut Self {
        let stage_flags = stages
            .iter()
            .map(|s| s.stage())
            .fold(vk::ShaderStageFlags::empty(), |a, b| a | b);

        self.bindings.push(
            vk::DescriptorSetLayoutBinding::builder()
                .binding(self.binding_id)
                .descriptor_type(desc_type)
                .descriptor_count(1)
                .stage_flags(stage_flags)
                .build(),
        );

        *self.descriptor_counts.entry(desc_type).or_insert(0) += 1;
        self.variables.push((desc_type, variable));

        self.binding_id += 1;
        self
    }

    /// load and crate a shader
    pub fn real_shader(&mut self, shader_type: ShaderType, name: &str) -> &mut Self {
        let shader = ShaderModule::new(
            Arc::clone(&self.context),
            format!("{}/{}", self.folder, name),
        );

        self.shaders
            .push((self.shader_group_id, shader_type, Some(shader)));
        self.shader_group_id += 1;
        self
    }

    /// create a `fake` shader; the shader won't be called
    pub fn fake_shader(&mut self, shader_type: ShaderType) -> &mut Self {
        if self.shader_group_id == 0 {
            panic!("can't create a fake shader when no other shader has been created");
        }

        self.shaders
            .push((self.shader_group_id - 1, shader_type, None));
        self
    }

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

    pub fn build(&mut self, max_recursion_depth: u32) -> RaytracerPipeline {
        /////////// CREATE PIPELINE LAYOUT
        let descriptor_set_layout = unsafe {
            self.context
                .device()
                .create_descriptor_set_layout(
                    &vk::DescriptorSetLayoutCreateInfo::builder().bindings(&self.bindings),
                    None,
                )
                .expect("Failed to create descriptor set layout")
        };

        let descriptor_set_layouts = [descriptor_set_layout];

        let pipeline_layout_create_info =
            vk::PipelineLayoutCreateInfo::builder().set_layouts(&descriptor_set_layouts);
        let pipeline_layout = unsafe {
            self.context
                .device()
                .create_pipeline_layout(&pipeline_layout_create_info, None)
                .expect("Failed to create pipeline layout")
        };

        /////////// CREATE PIPELINE
        let stages = self.stages();
        let groups = self.groups();

        let pipeline_create_info = [vk::RayTracingPipelineCreateInfoNV::builder()
            .stages(&stages)
            .groups(&groups)
            .max_recursion_depth(max_recursion_depth)
            .layout(pipeline_layout)
            .build()];

        let pipeline = unsafe {
            self.context
                .ray_tracing()
                .create_ray_tracing_pipelines(
                    vk::PipelineCache::null(),
                    &pipeline_create_info,
                    None,
                )
                .expect("Failed to create pipeline")[0]
        };

        /////////// CREATE BINDING TABLE
        let rt_properties = unsafe {
            RayTracing::get_properties(self.context.instance(), self.context.physical_device())
        };

        let shader_group_handle_size = rt_properties.shader_group_handle_size;
        let stb_size = shader_group_handle_size * 5;

        let mut shader_handles = Vec::new();
        shader_handles.resize(stb_size as _, 0u8);
        unsafe {
            self.context
                .ray_tracing()
                .get_ray_tracing_shader_group_handles(pipeline, 0, 5, &mut shader_handles)
                .expect("Failed to get rt shader group handles")
        };

        let shader_binding_table_buffer = BufferVariable::device_buffer(
            "shader_binding_table".to_string(),
            &self.context,
            vk::BufferUsageFlags::RAY_TRACING_NV,
            &shader_handles,
        )
        .0;

        /////////// CREATE DESCRIPTORS
        let descriptor_pool = self.create_descriptor_pool();
        let device = self.context.device();

        let descriptor_sets = {
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

            sets
        };

        RaytracerPipeline {
            rt_properties,
            pipeline,
            pipeline_layout,
            shader_binding_table_buffer,
            descriptor_set_layout,
            descriptor_pool,
            descriptor_sets,
            desc_types: self.variables.iter().map(|(a, _)| *a).collect(),
            context: Arc::clone(&self.context),
        }
    }
}
