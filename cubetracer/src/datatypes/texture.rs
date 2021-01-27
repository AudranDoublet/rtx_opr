use ash::{version::DeviceV1_0, vk};
use std::sync::Arc;

use crate::datatypes::*;
use crate::window::Swapchain;

use crate::context::Context;

pub struct TextureVariable {
    context: Arc<Context>,
    pub image: ImageVariable,
    pub view: vk::ImageView,
    sampler: Option<vk::Sampler>,

    info: Vec<vk::DescriptorImageInfo>,
}

impl TextureVariable {
    pub fn new(
        context: Arc<Context>,
        image: ImageVariable,
        view: vk::ImageView,
        sampler: Option<vk::Sampler>,
    ) -> Self {
        TextureVariable {
            context,
            image,
            view,
            sampler,
            info: Vec::new(),
        }
    }

    pub fn create_color_texture(
        context: &Arc<Context>,
        format: vk::Format,
        extent: vk::Extent2D,
        msaa_samples: vk::SampleCountFlags,
    ) -> TextureVariable {
        let image = ImageVariable::create(
            Arc::clone(context),
            ImageParameters {
                mem_properties: vk::MemoryPropertyFlags::DEVICE_LOCAL,
                extent,
                sample_count: msaa_samples,
                format,
                usage: vk::ImageUsageFlags::TRANSIENT_ATTACHMENT
                    | vk::ImageUsageFlags::COLOR_ATTACHMENT,
                ..Default::default()
            },
        );

        context.execute_one_time_commands(|cmd| {
            image.cmd_transition_image_layout(
                cmd,
                vk::ImageLayout::UNDEFINED,
                vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
            )
        });

        let view = image.create_view(vk::ImageViewType::TYPE_2D, vk::ImageAspectFlags::COLOR);

        TextureVariable::new(Arc::clone(context), image, view, None)
    }

    /// Create the depth buffer texture (image, memory and view).
    ///
    /// This function also transitions the image to be ready to be used
    /// as a depth/stencil attachement.
    pub fn create_depth_texture(
        context: &Arc<Context>,
        format: vk::Format,
        extent: vk::Extent2D,
        msaa_samples: vk::SampleCountFlags,
    ) -> TextureVariable {
        let image = ImageVariable::create(
            Arc::clone(context),
            ImageParameters {
                mem_properties: vk::MemoryPropertyFlags::DEVICE_LOCAL,
                extent,
                sample_count: msaa_samples,
                format,
                usage: vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT,
                ..Default::default()
            },
        );

        context.execute_one_time_commands(|cmd| {
            image.cmd_transition_image_layout(
                cmd,
                vk::ImageLayout::UNDEFINED,
                vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
            )
        });

        let view = image.create_view(vk::ImageViewType::TYPE_2D, vk::ImageAspectFlags::DEPTH);

        TextureVariable::new(Arc::clone(context), image, view, None)
    }

    pub fn texture_array2d(
        context: &Arc<Context>,
        width: u32,
        height: u32,
        paths: &Vec<String>,
    ) -> TextureVariable {
        let image = ImageVariable::array_from_paths(context, width, height, paths);
        context.execute_one_time_commands(|cmd| image.cmd_generate_mipmaps(cmd));

        let view = image.create_view(
            vk::ImageViewType::TYPE_2D_ARRAY,
            vk::ImageAspectFlags::COLOR,
        );
        let sampler = image.create_sampler_nearest(context);

        TextureVariable::new(Arc::clone(context), image, view, Some(sampler))
    }

    pub fn cmd_from_rgba(
        context: &Arc<Context>,
        command_buffer: vk::CommandBuffer,
        width: u32,
        height: u32,
        data: &[u8],
    ) -> (Self, BufferVariable) {
        let max_mip_levels = ((width.min(height) as f32).log2().floor() + 1.0) as u32;
        let extent = vk::Extent2D { width, height };
        let device = context.device();

        let buffer = BufferVariable::host_buffer(
            "rgba_texture_buffer".to_string(),
            context,
            vk::BufferUsageFlags::TRANSFER_SRC,
            data,
        );

        let image = ImageVariable::create(
            Arc::clone(context),
            ImageParameters {
                extent,
                mem_properties: vk::MemoryPropertyFlags::DEVICE_LOCAL,
                format: vk::Format::R8G8B8A8_UNORM,
                mip_levels: max_mip_levels,
                usage: vk::ImageUsageFlags::TRANSFER_SRC
                    | vk::ImageUsageFlags::TRANSFER_DST
                    | vk::ImageUsageFlags::SAMPLED,
                ..Default::default()
            },
        );

        // Transition the image layout and copy the buffer into the image
        // and transition the layout again to be readable from fragment shader.
        {
            image.cmd_transition_image_layout(
                command_buffer,
                vk::ImageLayout::UNDEFINED,
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            );

            image.cmd_copy_buffer(command_buffer, &buffer, extent);
            image.cmd_generate_mipmaps(command_buffer);
        }

        let image_view = image.create_view(vk::ImageViewType::TYPE_2D, vk::ImageAspectFlags::COLOR);

        let sampler = {
            let sampler_info = vk::SamplerCreateInfo::builder()
                .mag_filter(vk::Filter::LINEAR)
                .min_filter(vk::Filter::LINEAR)
                .address_mode_u(vk::SamplerAddressMode::REPEAT)
                .address_mode_v(vk::SamplerAddressMode::REPEAT)
                .address_mode_w(vk::SamplerAddressMode::REPEAT)
                .anisotropy_enable(true)
                .max_anisotropy(16.0)
                .border_color(vk::BorderColor::INT_OPAQUE_BLACK)
                .unnormalized_coordinates(false)
                .compare_enable(false)
                .compare_op(vk::CompareOp::ALWAYS)
                .mipmap_mode(vk::SamplerMipmapMode::LINEAR)
                .mip_lod_bias(0.0)
                .min_lod(0.0)
                .max_lod(max_mip_levels as _);

            unsafe {
                device
                    .create_sampler(&sampler_info, None)
                    .expect("Failed to create sampler")
            }
        };

        let texture = TextureVariable::new(Arc::clone(context), image, image_view, Some(sampler));

        (texture, buffer)
    }

    pub fn create_default_sampler(context: &Arc<Context>,) -> vk::Sampler {
        let device = context.device();
        let sampler_info = vk::SamplerCreateInfo::builder()
                .mag_filter(vk::Filter::LINEAR)
                .min_filter(vk::Filter::LINEAR)
                .address_mode_u(vk::SamplerAddressMode::CLAMP_TO_EDGE)
                .address_mode_v(vk::SamplerAddressMode::CLAMP_TO_EDGE)
                .address_mode_w(vk::SamplerAddressMode::CLAMP_TO_EDGE)
                .anisotropy_enable(false)
                .max_anisotropy(0.0)
                .border_color(vk::BorderColor::FLOAT_OPAQUE_WHITE)
                .unnormalized_coordinates(false)
                .compare_enable(false)
                .compare_op(vk::CompareOp::ALWAYS)
                .mipmap_mode(vk::SamplerMipmapMode::LINEAR)
                .mip_lod_bias(0.0)
                .min_lod(0.0)
                .max_lod(1.0);

        unsafe {
            device
                .create_sampler(&sampler_info, None)
                .expect("Failed to create sampler")
        }
    }

    pub fn create_renderable_texture(
        context: &Arc<Context>,
        width: u32,
        height: u32,
        format: vk::Format,
    ) -> Self {
        let extent = vk::Extent2D { width, height };

        let image = ImageVariable::create(
            Arc::clone(context),
            ImageParameters {
                mem_properties: vk::MemoryPropertyFlags::DEVICE_LOCAL,
                extent,
                format,
                usage: vk::ImageUsageFlags::SAMPLED | vk::ImageUsageFlags::COLOR_ATTACHMENT,
                ..Default::default()
            },
        );

        context.execute_one_time_commands(|exec| {
            image.cmd_transition_image_layout(
                exec,
                vk::ImageLayout::UNDEFINED,
                vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
            )
        });

        let image_view = image.create_view(vk::ImageViewType::TYPE_2D, vk::ImageAspectFlags::COLOR);

        let sampler = Self::create_default_sampler(context);

        TextureVariable::new(Arc::clone(context), image, image_view, Some(sampler))
    }

    pub fn from_swapchain(context: &Arc<Context>, swapchain: &Swapchain) -> TextureVariable {
        let swapchain_props = swapchain.properties();

        let params = ImageParameters {
            mem_properties: vk::MemoryPropertyFlags::DEVICE_LOCAL,
            extent: swapchain_props.extent,
            format: swapchain_props.format.format,
            usage: vk::ImageUsageFlags::TRANSFER_SRC 
                | vk::ImageUsageFlags::STORAGE 
                | vk::ImageUsageFlags::TRANSFER_DST,
            ..Default::default()
        };
        let image = ImageVariable::create(Arc::clone(context), params);
        let view = image.create_view(vk::ImageViewType::TYPE_2D, vk::ImageAspectFlags::COLOR);

        context.execute_one_time_commands(|ctx| {
            image.cmd_transition_image_layout(
                ctx,
                vk::ImageLayout::UNDEFINED,
                vk::ImageLayout::GENERAL,
            )
        });

        TextureVariable {
            context: Arc::clone(context),
            image,
            view,
            sampler: None,
            info: Vec::new(),
        }
    }

    pub fn from_extent(
        context: &Arc<Context>,
        extent: vk::Extent2D,
        format: vk::Format,
    ) -> TextureVariable {
        let params = ImageParameters {
            mem_properties: vk::MemoryPropertyFlags::DEVICE_LOCAL,
            extent: extent,
            format,
            usage: vk::ImageUsageFlags::TRANSFER_DST
                | vk::ImageUsageFlags::TRANSFER_SRC
                | vk::ImageUsageFlags::STORAGE,
            ..Default::default()
        };
        let image = ImageVariable::create(Arc::clone(context), params);
        let view = image.create_view(vk::ImageViewType::TYPE_2D, vk::ImageAspectFlags::COLOR);

        context.execute_one_time_commands(|ctx| {
            image.cmd_transition_image_layout(
                ctx,
                vk::ImageLayout::UNDEFINED,
                vk::ImageLayout::GENERAL,
            )
        });

        TextureVariable {
            context: Arc::clone(context),
            image,
            view,
            sampler: None,
            info: Vec::new(),
        }
    }

    pub fn from_swapchain_format(
        context: &Arc<Context>,
        swapchain: &Swapchain,
        format: vk::Format,
    ) -> TextureVariable {
        let swapchain_props = swapchain.properties();
        Self::from_extent(context, swapchain_props.extent, format)
    }
}

impl TextureVariable {
    pub fn set_sampler(&mut self, sampler: vk::Sampler) {
        self.sampler = Some(sampler);
    }

    pub fn fill_image<T: Sized + Copy>(&self, context: &Arc<Context>, data: &[T]) {
        let buffer = BufferVariable::host_buffer(
            "texture_buffer".to_string(),
            &self.context,
            vk::BufferUsageFlags::TRANSFER_SRC,
            data,
        );

        // Transition the image layout and copy the buffer into the image
        // and transition the layout again to be readable from fragment shader.
        context.execute_one_time_commands(|cmd| {
            self.image.cmd_transition_image_layout(
                cmd,
                vk::ImageLayout::UNDEFINED,
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            );

            self.image.cmd_copy_buffer(
                cmd,
                &buffer,
                self.image.extent()
            );
        });
    }
}

impl DataType for TextureVariable {
    fn write_descriptor_builder(&mut self) -> vk::WriteDescriptorSetBuilder {
        self.info = vec![{
            let bld = vk::DescriptorImageInfo::builder()
                .image_view(self.view)
                .image_layout(vk::ImageLayout::GENERAL);

            if let Some(sampler) = self.sampler {
                bld.sampler(sampler)
            } else {
                bld
            }
            .build()
        }];

        vk::WriteDescriptorSet::builder().image_info(&self.info)
    }

    fn len(&self) -> usize {
        1
    }
}

impl Drop for TextureVariable {
    fn drop(&mut self) {
        unsafe {
            if let Some(sampler) = self.sampler.take() {
                self.context.device().destroy_sampler(sampler, None);
            }
            self.context.device().destroy_image_view(self.view, None);
        }
    }
}
