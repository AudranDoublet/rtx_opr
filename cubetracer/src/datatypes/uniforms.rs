use crate::context::Context;
use ash::vk;
use nalgebra::{Vector3, Vector4, Matrix4};
use std::sync::Arc;

use crate::datatypes::{BufferVariable, DataType};

#[repr(C)]
// vec3 take 4 float in glsl, so use rust's vec4
pub struct UniformCamera {
    pub screen_to_world: Matrix4<f64>,
    pub prev_world_to_screen: Matrix4<f64>,
    pub origin: Vector4<f64>,
}

#[repr(C)]
pub struct UniformSun {
    pub projection: Matrix4<f32>,
    pub direction: Vector4<f32>,
}

#[repr(C)]
pub struct UniformScene {
    pub rendered_buffer: u32,
}

pub struct UniformVariable {
    host_buffer: BufferVariable,
    device_buffer: BufferVariable,
    info: Vec<vk::DescriptorBufferInfo>,
}

impl UniformVariable {
    pub fn new<T: Sized>(context: &Arc<Context>, value: &T) -> UniformVariable {
        let data = unsafe { any_as_u8_slice(value) };

        let (device_buffer, host_buffer) = BufferVariable::device_buffer(
            "uniform_variable_buffer".to_string(),
            context,
            vk::BufferUsageFlags::UNIFORM_BUFFER,
            data,
        );

        let mut variable = UniformVariable {
            host_buffer,
            device_buffer,
            info: Vec::new(),
        };

        variable.set(context, value);
        variable
    }

    pub fn set<T: Sized>(&mut self, context: &Arc<Context>, value: &T) {
        let data = unsafe { any_as_u8_slice(value) };

        self.host_buffer.set_host(data);

        context.execute_one_time_commands(|command_buffer| {
            self.device_buffer
                .cmd_copy(command_buffer, &self.host_buffer, self.host_buffer.size());
        });
    }

    pub fn buffer(&self) -> &BufferVariable {
        &self.device_buffer
    }
}

impl DataType for UniformVariable {
    fn write_descriptor_builder(&mut self) -> vk::WriteDescriptorSetBuilder {
        self.info = vec![vk::DescriptorBufferInfo::builder()
            .buffer(*self.device_buffer.buffer())
            .range(vk::WHOLE_SIZE)
            .build()];

        vk::WriteDescriptorSet::builder().buffer_info(&self.info)
    }

    fn len(&self) -> usize {
        1
    }
}

pub unsafe fn any_as_u8_slice<T: Sized>(any: &T) -> &[u8] {
    let ptr = (any as *const T) as *const u8;
    std::slice::from_raw_parts(ptr, std::mem::size_of::<T>())
}
