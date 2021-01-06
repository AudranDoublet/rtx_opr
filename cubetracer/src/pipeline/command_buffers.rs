use std::sync::Arc;

use crate::window::*;
use crate::context::*;

use ash::vk;
use ash::version::DeviceV1_0;

pub struct CommandBuffers {
    context: Arc<Context>,
    command_buffers: Vec<vk::CommandBuffer>
}

impl CommandBuffers {
    pub fn new(context: &Arc<Context>, swapchain: &Swapchain) -> Self {
        let device = context.device();
        let image_count = swapchain.image_count();

        let allocate_info = vk::CommandBufferAllocateInfo::builder()
            .command_pool(context.general_command_pool())
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(image_count as _);

        let command_buffers = unsafe {
            device
                .allocate_command_buffers(&allocate_info)
                .expect("Failed to allocate command buffers")
        };

        CommandBuffers {
            context: Arc::clone(context),
            command_buffers,
        }
    }
}

impl CommandBuffers {
    pub fn buffers(&self) -> &Vec<vk::CommandBuffer> {
        &self.command_buffers
    }

    pub fn record<F>(&self, func: F) where F: Fn(usize, vk::CommandBuffer) {
        let command_buffer_begin_info = vk::CommandBufferBeginInfo::builder()
            .flags(vk::CommandBufferUsageFlags::SIMULTANEOUS_USE);

        let device = self.context.device();

        self.command_buffers
            .iter()
            .enumerate()
            .for_each(|(index, buffer)| {
                let buffer = *buffer;

                unsafe {
                    device
                        .begin_command_buffer(buffer, &command_buffer_begin_info)
                        .expect("Failed to begin command buffer")
                };

                func(index, buffer);

                unsafe {
                    device
                        .end_command_buffer(buffer)
                        .expect("Failed to end command buffer")
                };
            });
    }
}
