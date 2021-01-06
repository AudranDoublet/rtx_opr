use ash::extensions::nv::RayTracing;
use ash::version::DeviceV1_0;
use ash::vk;

use crate::context::Context;
use std::ffi::CString;
use std::sync::Arc;

use crate::datatypes::*;
use crate::descriptors::*;
use crate::pipeline::*;

enum ShaderGroup {
    Hit {
        any_hit: Option<u32>,
        closest_hit: Option<u32>,
        intersection: Option<u32>,
    },
    General(u32),
}

impl ShaderGroup {
    pub fn group(&self) -> vk::RayTracingShaderGroupTypeNV {
        match self {
            ShaderGroup::General(..) => vk::RayTracingShaderGroupTypeNV::GENERAL,
            ShaderGroup::Hit { .. } => vk::RayTracingShaderGroupTypeNV::TRIANGLES_HIT_GROUP,
        }
    }
}

trait BuilderWithType {
    fn shader_with_type(self, s_type: &ShaderGroup) -> Self;
}

impl<'a> BuilderWithType for vk::RayTracingShaderGroupCreateInfoNVBuilder<'a> {
    fn shader_with_type(self, s_type: &ShaderGroup) -> Self {
        match s_type {
            ShaderGroup::Hit {
                any_hit,
                closest_hit,
                intersection,
            } => self
                .general_shader(vk::SHADER_UNUSED_NV)
                .closest_hit_shader(closest_hit.unwrap_or(vk::SHADER_UNUSED_NV))
                .any_hit_shader(any_hit.unwrap_or(vk::SHADER_UNUSED_NV))
                .intersection_shader(intersection.unwrap_or(vk::SHADER_UNUSED_NV)),

            ShaderGroup::General(i) => self
                .general_shader(*i)
                .closest_hit_shader(vk::SHADER_UNUSED_NV)
                .any_hit_shader(vk::SHADER_UNUSED_NV)
                .intersection_shader(vk::SHADER_UNUSED_NV),
        }
    }
}

pub struct PipelineBuilder {
    folder: String,

    context: Arc<Context>,
    entry_point: CString,

    descriptor_sets: Vec<vk::DescriptorSet>,
    descriptor_set_layouts: Vec<vk::DescriptorSetLayout>,

    miss_offset: usize,
    hit_offset: usize,

    shaders: Vec<(ShaderType, ShaderModule)>,
    shader_groups: Vec<ShaderGroup>,
}

impl PipelineBuilder {
    pub fn new(context: &Arc<Context>, folder: &str) -> PipelineBuilder {
        PipelineBuilder {
            context: Arc::clone(context),
            folder: folder.to_string(),

            shaders: Vec::new(),
            shader_groups: Vec::new(),

            descriptor_set_layouts: Vec::new(),
            descriptor_sets: Vec::new(),
            miss_offset: 0,
            hit_offset: 0,

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
            None => Some(self.shaders.len() as u32 - 2),
        };

        let any_hit = match any_hit {
            Some(name) => Some(self.load_shader(ShaderType::AnyHit, name)),
            None => None,
        };

        self.shader_groups.push(ShaderGroup::Hit {
            any_hit,
            closest_hit,
            intersection: None,
        });

        self
    }

    pub fn build(&mut self, max_recursion_depth: u32) -> RaytracerPipeline {
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
                .get_ray_tracing_shader_group_handles(
                    pipeline,
                    0,
                    self.shader_groups.len() as u32,
                    &mut shader_handles,
                )
                .expect("Failed to get rt shader group handles")
        };

        let shader_binding_table_buffer = BufferVariable::device_buffer(
            "shader_binding_table".to_string(),
            &self.context,
            vk::BufferUsageFlags::RAY_TRACING_NV,
            &shader_handles,
        )
        .0;

        RaytracerPipeline {
            rt_properties,
            pipeline,
            pipeline_layout,
            shader_binding_table_buffer,
            descriptor_sets: self.descriptor_sets.clone(),
            context: Arc::clone(&self.context),
            miss_offset: self.miss_offset,
            hit_offset: self.hit_offset,
        }
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
