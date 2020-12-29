use ash::extensions::nv::RayTracing;
use ash::version::DeviceV1_0;
use ash::vk;

use crate::context::Context;
use std::ffi::CString;
use std::sync::Arc;

use crate::datatypes::*;
use crate::pipeline::*;

use std::collections::HashMap;

enum ShaderGroup {
    Hit {
        any_hit: Option<u32>,
        closest_hit: Option<u32>,
        intersection: Option<u32>,
    },
    General (u32),
}

impl ShaderGroup {
    pub fn group(&self) -> vk::RayTracingShaderGroupTypeNV {
        match self {
            ShaderGroup::General (..) => vk::RayTracingShaderGroupTypeNV::GENERAL,
            ShaderGroup::Hit {..} => vk::RayTracingShaderGroupTypeNV::TRIANGLES_HIT_GROUP,
        }
    }
}

trait BuilderWithType {
    fn shader_with_type(self, s_type: &ShaderGroup) -> Self;
}

impl<'a> BuilderWithType for vk::RayTracingShaderGroupCreateInfoNVBuilder<'a> {
    fn shader_with_type(self, s_type: &ShaderGroup) -> Self {
        match s_type {
            ShaderGroup::Hit { any_hit, closest_hit, intersection } => self
                .general_shader(vk::SHADER_UNUSED_NV)
                .closest_hit_shader(closest_hit.unwrap_or(vk::SHADER_UNUSED_NV))
                .any_hit_shader(any_hit.unwrap_or(vk::SHADER_UNUSED_NV))
                .intersection_shader(intersection.unwrap_or(vk::SHADER_UNUSED_NV)),

            ShaderGroup::General (i) => self
                .general_shader(*i)
                .closest_hit_shader(vk::SHADER_UNUSED_NV)
                .any_hit_shader(vk::SHADER_UNUSED_NV)
                .intersection_shader(vk::SHADER_UNUSED_NV),
        }
    }
}

pub struct PipelineBuilder<'a> {
    folder: String,

    context: Arc<Context>,
    entry_point: CString,

    variables: Vec<(vk::DescriptorType, &'a mut dyn DataType)>,
    descriptor_counts: HashMap<vk::DescriptorType, u32>,
    bindings: Vec<vk::DescriptorSetLayoutBinding>,

    miss_offset: usize,
    hit_offset: usize,

    shaders: Vec<(ShaderType, ShaderModule)>,
    shader_groups: Vec<ShaderGroup>,
}

impl<'a> PipelineBuilder<'a> {
    pub fn new(context: &Arc<Context>, folder: &str) -> PipelineBuilder<'a> {
        PipelineBuilder {
            context: Arc::clone(context),
            folder: folder.to_string(),

            descriptor_counts: HashMap::new(),
            bindings: Vec::new(),
            shaders: Vec::new(),
            shader_groups: Vec::new(),
            variables: Vec::new(),

            miss_offset: 0,
            hit_offset: 0,

            entry_point: CString::new("main").unwrap(),
        }
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

    /// retrieve created raytracing groups
    fn groups(&self) -> Vec<vk::RayTracingShaderGroupCreateInfoNV> {
        let mut groups = Vec::new();

        for shader_group in &self.shader_groups {
            groups.push(
                vk::RayTracingShaderGroupCreateInfoNV::builder()
                    .ty(shader_group.group())
                    .shader_with_type(shader_group)
                    .build(),
            );
        }

        groups
    }

    /// add a new memory binding for shaders
    pub fn binding(
        &mut self,
        idx: u32,
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
                .binding(idx)
                .descriptor_type(desc_type)
                .descriptor_count(descriptor_count)
                .stage_flags(stage_flags)
                .build(),
        );

        *self.descriptor_counts.entry(desc_type).or_insert(0) += 1;
        self.variables.push((desc_type, variable));

        self
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
    pub fn general_shader(&mut self, shader_type: ShaderType, name: &str) -> &mut Self {
        if shader_type == ShaderType::Miss && self.miss_offset == 0 {
            self.miss_offset = self.shader_groups.len();
        }

        let shader_id = self.load_shader(shader_type, name);

        self.shader_groups.push(ShaderGroup::General(shader_id));
        self
    }

    pub fn hit_shaders(&mut self, closest_hit: Option<&str>, any_hit: Option<&str>) -> &mut Self {
        if self.hit_offset == 0 {
            self.hit_offset = self.shader_groups.len();
        }

        let closest_hit = match closest_hit {
            Some(name) => Some(self.load_shader(ShaderType::ClosestHit, name)),
            None => Some(self.shaders.len() as u32 - 2)
        };

        let any_hit = match any_hit {
            Some(name) => Some(self.load_shader(ShaderType::AnyHit, name)),
            None => None
        };

        self.shader_groups.push(ShaderGroup::Hit {
            any_hit,
            closest_hit,
            intersection: None,
        });

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
        let stb_size = shader_group_handle_size * self.shader_groups.len() as u32;

        let mut shader_handles = Vec::new();
        shader_handles.resize(stb_size as _, 0u8);
        unsafe {
            self.context
                .ray_tracing()
                .get_ray_tracing_shader_group_handles(pipeline, 0, self.shader_groups.len() as u32, &mut shader_handles)
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
            miss_offset: self.miss_offset,
            hit_offset: self.hit_offset,
        }
    }
}
