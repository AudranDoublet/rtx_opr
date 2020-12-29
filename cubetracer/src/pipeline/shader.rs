use crate::context::Context;
use ash::{version::DeviceV1_0, vk};
use std::{path::Path, sync::Arc};

#[derive(Copy, Clone, PartialEq)]
pub enum ShaderType {
    Raygen,
    ClosestHit,
    AnyHit,
    Miss,
}

impl ShaderType {
    pub fn stage(&self) -> vk::ShaderStageFlags {
        match self {
            ShaderType::Raygen => vk::ShaderStageFlags::RAYGEN_NV,
            ShaderType::ClosestHit => vk::ShaderStageFlags::CLOSEST_HIT_NV,
            ShaderType::AnyHit => vk::ShaderStageFlags::ANY_HIT_NV,
            ShaderType::Miss => vk::ShaderStageFlags::MISS_NV,
        }
    }

    pub fn group(&self) -> vk::RayTracingShaderGroupTypeNV {
        match self {
            ShaderType::Raygen => vk::RayTracingShaderGroupTypeNV::GENERAL,
            ShaderType::ClosestHit => vk::RayTracingShaderGroupTypeNV::TRIANGLES_HIT_GROUP,
            ShaderType::AnyHit => vk::RayTracingShaderGroupTypeNV::TRIANGLES_HIT_GROUP,
            ShaderType::Miss => vk::RayTracingShaderGroupTypeNV::GENERAL,
        }
    }
}

pub struct ShaderModule {
    context: Arc<Context>,
    module: vk::ShaderModule,
}

impl ShaderModule {
    pub fn new<P: AsRef<Path>>(context: Arc<Context>, path: P) -> Self {
        let source = {
            log::debug!("Loading shader file {}", path.as_ref().to_str().unwrap());
            let mut file = std::fs::File::open(path).expect("Failed to open shader file");
            ash::util::read_spv(&mut file).expect("Failed to read shader source")
        };
        let module = {
            let create_info = vk::ShaderModuleCreateInfo::builder().code(&source);
            unsafe {
                context
                    .device()
                    .create_shader_module(&create_info, None)
                    .expect("Failed to create shader module")
            }
        };
        Self { context, module }
    }
}

impl ShaderModule {
    pub fn module(&self) -> vk::ShaderModule {
        self.module
    }
}

impl Drop for ShaderModule {
    fn drop(&mut self) {
        let device = self.context.device();
        unsafe { device.destroy_shader_module(self.module, None) };
    }
}
