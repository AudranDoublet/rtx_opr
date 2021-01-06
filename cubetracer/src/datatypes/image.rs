extern crate image;

use ash::vk;
use std::sync::Arc;

use crate::context::{find_memory_type, Context};

use crate::window::SwapchainProperties;

use ash::version::{DeviceV1_0, InstanceV1_0};

use crate::datatypes::*;

#[derive(Copy, Clone)]
pub struct ImageParameters {
    pub mem_properties: vk::MemoryPropertyFlags,
    pub extent: vk::Extent2D,
    pub layers: u32,
    pub mip_levels: u32,
    pub sample_count: vk::SampleCountFlags,
    pub format: vk::Format,
    pub tiling: vk::ImageTiling,
    pub usage: vk::ImageUsageFlags,
    pub create_flags: vk::ImageCreateFlags,
}

impl Default for ImageParameters {
    fn default() -> Self {
        Self {
            mem_properties: vk::MemoryPropertyFlags::empty(),
            extent: vk::Extent2D {
                width: 0,
                height: 0,
            },
            layers: 1,
            mip_levels: 1,
            sample_count: vk::SampleCountFlags::TYPE_1,
            format: vk::Format::R8G8B8A8_UNORM,
            tiling: vk::ImageTiling::OPTIMAL,
            usage: vk::ImageUsageFlags::SAMPLED,
            create_flags: vk::ImageCreateFlags::empty(),
        }
    }
}

pub struct ImageVariable {
    context: Arc<Context>,
    pub image: vk::Image,
    memory: Option<vk::DeviceMemory>,
    extent: vk::Extent3D,
    format: vk::Format,
    mip_levels: u32,
    layers: u32,
    managed: bool,
}

impl ImageVariable {
    pub fn create(context: Arc<Context>, parameters: ImageParameters) -> Self {
        let extent = vk::Extent3D {
            width: parameters.extent.width,
            height: parameters.extent.height,
            depth: 1,
        };

        let image_info = vk::ImageCreateInfo::builder()
            .image_type(vk::ImageType::TYPE_2D)
            .extent(extent)
            .mip_levels(parameters.mip_levels)
            .array_layers(parameters.layers)
            .format(parameters.format)
            .tiling(parameters.tiling)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .usage(parameters.usage)
            .sharing_mode(vk::SharingMode::EXCLUSIVE)
            .samples(parameters.sample_count)
            .flags(parameters.create_flags);

        let device = context.device();
        let image = unsafe {
            device
                .create_image(&image_info, None)
                .expect("Failed to create image")
        };
        let mem_requirements = unsafe { device.get_image_memory_requirements(image) };
        let mem_type_index = find_memory_type(
            mem_requirements,
            context.get_mem_properties(),
            parameters.mem_properties,
        );

        let alloc_info = vk::MemoryAllocateInfo::builder()
            .allocation_size(mem_requirements.size)
            .memory_type_index(mem_type_index);

        let memory = unsafe {
            let mem = device
                .allocate_memory(&alloc_info, None)
                .expect("Failed to allocate image memory");
            device
                .bind_image_memory(image, mem, 0)
                .expect("Failed to bind image memory");
            mem
        };

        Self {
            context,
            image,
            memory: Some(memory),
            extent,
            format: parameters.format,
            mip_levels: parameters.mip_levels,
            layers: parameters.layers,
            managed: false,
        }
    }

    pub fn from_path(context: &Arc<Context>, path: &str, width: u32, height: u32) -> ImageVariable {
        let image = image::open(path)
            .expect(format!("can't load texture {}", path).as_str())
            .into_rgba8();

        let rimage =
            image::imageops::resize(&image, width, height, image::imageops::FilterType::Gaussian);

        let params = ImageParameters {
            extent: vk::Extent2D { width, height },
            layers: 1,
            mip_levels: 1,
            mem_properties: vk::MemoryPropertyFlags::DEVICE_LOCAL,
            format: vk::Format::R8G8B8A8_UNORM,
            usage: vk::ImageUsageFlags::TRANSFER_SRC
                | vk::ImageUsageFlags::TRANSFER_DST
                | vk::ImageUsageFlags::SAMPLED,
            ..ImageParameters::default()
        };

        let buffer = BufferVariable::host_buffer(
            "image_data".to_string(),
            context,
            vk::BufferUsageFlags::TRANSFER_SRC,
            &rimage.into_raw(),
        );

        let image = Self::create(Arc::clone(context), params);
        context.execute_one_time_commands(|cmd| {
            image.cmd_transition_image_layout(
                cmd,
                vk::ImageLayout::UNDEFINED,
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            );

            image.cmd_copy_buffer(cmd, &buffer, vk::Extent2D { width, height });
            image.cmd_transition_image_layout(
                cmd,
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
            );
        });

        image
    }

    pub fn array_from_paths(
        context: &Arc<Context>,
        width: u32,
        height: u32,
        paths: &Vec<String>,
    ) -> ImageVariable {
        let images = paths
            .iter()
            .map(|path| Self::from_path(context, path, width, height))
            .collect();

        Self::create_image_array(context, width, height, images)
    }

    pub fn create_image_array(
        context: &Arc<Context>,
        width: u32,
        height: u32,
        images: Vec<ImageVariable>,
    ) -> Self {
        // compute needed mip levels
        let mip_levels = (width.max(height) as f32).log2().floor() as u32 + 1;
        dbg!(mip_levels);

        let params = ImageParameters {
            extent: vk::Extent2D { width, height },
            layers: images.len() as u32,
            mip_levels,
            mem_properties: vk::MemoryPropertyFlags::DEVICE_LOCAL,
            format: vk::Format::R8G8B8A8_UNORM,
            usage: vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED,
            ..ImageParameters::default()
        };

        let result = Self::create(Arc::clone(context), params);
        context.execute_one_time_commands(|cmd_buffer| {
            result.cmd_transition_image_layout(
                cmd_buffer,
                vk::ImageLayout::UNDEFINED,
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            );
        });

        for (i, image) in images.iter().enumerate() {
            let regions = [vk::ImageCopy::builder()
                .src_subresource(vk::ImageSubresourceLayers {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    mip_level: 0,
                    base_array_layer: 0,
                    layer_count: 1,
                })
                .dst_subresource(vk::ImageSubresourceLayers {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    mip_level: 0,
                    base_array_layer: i as u32,
                    layer_count: 1,
                })
                .dst_offset(vk::Offset3D { x: 0, y: 0, z: 0 })
                .extent(image.extent)
                .build()];

            context.execute_one_time_commands(|cmd_buffer| unsafe {
                context.device().cmd_copy_image(
                    cmd_buffer,
                    image.image,
                    vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
                    result.image,
                    vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                    &regions,
                );
            });
        }

        context.execute_one_time_commands(|cmd_buffer| {
            result.cmd_transition_image_layout(
                cmd_buffer,
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                vk::ImageLayout::GENERAL,
            );
        });

        result
    }

    pub fn create_swapchain_image(
        context: Arc<Context>,
        image: vk::Image,
        swapchain_properties: SwapchainProperties,
    ) -> Self {
        Self {
            context,
            image,
            memory: None,
            extent: vk::Extent3D {
                width: swapchain_properties.extent.width,
                height: swapchain_properties.extent.height,
                depth: 1,
            },
            format: swapchain_properties.format.format,
            mip_levels: 1,
            layers: 1,
            managed: true,
        }
    }
}

impl ImageVariable {
    pub fn create_view(
        &self,
        view_type: vk::ImageViewType,
        aspect_mask: vk::ImageAspectFlags,
    ) -> vk::ImageView {
        let create_info = vk::ImageViewCreateInfo::builder()
            .image(self.image)
            .view_type(view_type)
            .format(self.format)
            .subresource_range(vk::ImageSubresourceRange {
                aspect_mask,
                base_mip_level: 0,
                level_count: self.mip_levels,
                base_array_layer: 0,
                layer_count: self.layers,
            });

        unsafe {
            self.context
                .device()
                .create_image_view(&create_info, None)
                .expect("Failed to create image view")
        }
    }

    pub fn create_sampler(&self, context: &Arc<Context>) -> vk::Sampler {
        let info = vk::SamplerCreateInfo::builder()
            .mag_filter(vk::Filter::LINEAR)
            .min_filter(vk::Filter::LINEAR)
            .mipmap_mode(vk::SamplerMipmapMode::LINEAR)
            .address_mode_u(vk::SamplerAddressMode::CLAMP_TO_EDGE)
            .address_mode_v(vk::SamplerAddressMode::CLAMP_TO_EDGE)
            .address_mode_w(vk::SamplerAddressMode::CLAMP_TO_EDGE)
            .mip_lod_bias(0.0)
            .min_lod(0.0)
            .max_lod(self.mip_levels as f32)
            .build();

        unsafe {
            context
                .device()
                .create_sampler(&info, None)
                .expect("can't create sampler")
        }
    }

    pub fn cmd_transition_image_layout(
        &self,
        command_buffer: vk::CommandBuffer,
        old_layout: vk::ImageLayout,
        new_layout: vk::ImageLayout,
    ) {
        let (src_access_mask, dst_access_mask, src_stage, dst_stage) =
            match (old_layout, new_layout) {
                (vk::ImageLayout::UNDEFINED, vk::ImageLayout::TRANSFER_DST_OPTIMAL) => (
                    vk::AccessFlags::empty(),
                    vk::AccessFlags::TRANSFER_WRITE,
                    vk::PipelineStageFlags::TOP_OF_PIPE,
                    vk::PipelineStageFlags::TRANSFER,
                ),
                (
                    vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                    vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
                ) => (
                    vk::AccessFlags::TRANSFER_WRITE,
                    vk::AccessFlags::SHADER_READ,
                    vk::PipelineStageFlags::TRANSFER,
                    vk::PipelineStageFlags::FRAGMENT_SHADER,
                ),
                (vk::ImageLayout::UNDEFINED, vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL) => {
                    (
                        vk::AccessFlags::empty(),
                        vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_READ
                            | vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE,
                        vk::PipelineStageFlags::TOP_OF_PIPE,
                        vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS,
                    )
                }
                (vk::ImageLayout::UNDEFINED, vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL) => (
                    vk::AccessFlags::empty(),
                    vk::AccessFlags::COLOR_ATTACHMENT_READ
                        | vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
                    vk::PipelineStageFlags::TOP_OF_PIPE,
                    vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
                ),
                (
                    vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
                    vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
                ) => (
                    vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
                    vk::AccessFlags::SHADER_READ,
                    vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
                    vk::PipelineStageFlags::FRAGMENT_SHADER,
                ),
                (
                    vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
                    vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                ) => (
                    vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
                    vk::AccessFlags::TRANSFER_WRITE,
                    vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
                    vk::PipelineStageFlags::TRANSFER,
                ),
                _ => (
                    vk::AccessFlags::empty(),
                    vk::AccessFlags::empty(),
                    vk::PipelineStageFlags::TOP_OF_PIPE,
                    vk::PipelineStageFlags::TOP_OF_PIPE,
                ),
            };

        let aspect_mask = if new_layout == vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL {
            let mut mask = vk::ImageAspectFlags::DEPTH;
            if has_stencil_component(self.format) {
                mask |= vk::ImageAspectFlags::STENCIL;
            }
            mask
        } else {
            vk::ImageAspectFlags::COLOR
        };

        let barrier = vk::ImageMemoryBarrier::builder()
            .old_layout(old_layout)
            .new_layout(new_layout)
            .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .image(self.image)
            .subresource_range(vk::ImageSubresourceRange {
                aspect_mask,
                base_mip_level: 0,
                level_count: self.mip_levels,
                base_array_layer: 0,
                layer_count: self.layers,
            })
            .src_access_mask(src_access_mask)
            .dst_access_mask(dst_access_mask)
            .build();
        let barriers = [barrier];

        unsafe {
            self.context.device().cmd_pipeline_barrier(
                command_buffer,
                src_stage,
                dst_stage,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &barriers,
            )
        };
    }

    pub fn cmd_copy_buffer(
        &self,
        command_buffer: vk::CommandBuffer,
        buffer: &BufferVariable,
        extent: vk::Extent2D,
    ) {
        let region = vk::BufferImageCopy::builder()
            .buffer_offset(0)
            .buffer_row_length(0)
            .buffer_image_height(0)
            .image_subresource(vk::ImageSubresourceLayers {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                mip_level: 0,
                base_array_layer: 0,
                layer_count: self.layers,
            })
            .image_offset(vk::Offset3D { x: 0, y: 0, z: 0 })
            .image_extent(vk::Extent3D {
                width: extent.width,
                height: extent.height,
                depth: 1,
            })
            .build();
        let regions = [region];
        unsafe {
            self.context.device().cmd_copy_buffer_to_image(
                command_buffer,
                *buffer.buffer(),
                self.image,
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                &regions,
            )
        }
    }

    /// create a BufferImageCopy for this image to a buffer
    pub fn buffer_image_copy(&self, offset: u64, layer: u32) -> vk::BufferImageCopy {
        vk::BufferImageCopy::builder()
            .image_subresource(vk::ImageSubresourceLayers {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                mip_level: 0,
                base_array_layer: layer,
                layer_count: 1,
            })
            .image_extent(vk::Extent3D {
                width: self.extent.width,
                height: self.extent.height,
                depth: 1,
            })
            .buffer_offset(offset)
            .build()
    }

    /// Record command to copy [src_image] into this image.
    ///
    /// The full extent of the passed in layer will be copied, so the target image
    /// should be big enough to contain the content of the source image.
    ///
    /// Source image layout should be TRANSFER_SRC_OPTIMAL and target TRANSFER_DST_OPTIMAL.
    pub fn cmd_copy(
        &self,
        command_buffer: vk::CommandBuffer,
        src_image: &Self,
        subresource_layers: vk::ImageSubresourceLayers,
    ) {
        let image_copy_info = [vk::ImageCopy::builder()
            .src_subresource(subresource_layers)
            .src_offset(vk::Offset3D { x: 0, y: 0, z: 0 })
            .dst_subresource(subresource_layers)
            .dst_offset(vk::Offset3D { x: 0, y: 0, z: 0 })
            .extent(src_image.extent)
            .build()];

        unsafe {
            self.context.device().cmd_copy_image(
                command_buffer,
                src_image.image,
                vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
                self.image,
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                &image_copy_info,
            );
        };
    }

    pub fn cmd_generate_mipmaps(&self, command_buffer: vk::CommandBuffer) {
        let format_properties = unsafe {
            self.context
                .instance()
                .get_physical_device_format_properties(self.context.physical_device(), self.format)
        };
        if !format_properties
            .optimal_tiling_features
            .contains(vk::FormatFeatureFlags::SAMPLED_IMAGE_FILTER_LINEAR)
        {
            panic!(
                "Linear blitting is not supported for format {:?}.",
                self.format
            )
        }

        let mut barrier = vk::ImageMemoryBarrier::builder()
            .image(self.image)
            .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .subresource_range(vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                base_array_layer: 0,
                layer_count: self.layers,
                level_count: 1,
                ..Default::default()
            })
            .build();

        let mut mip_width = self.extent.width as i32;
        let mut mip_height = self.extent.height as i32;
        for level in 1..self.mip_levels {
            let next_mip_width = if mip_width > 1 {
                mip_width / 2
            } else {
                mip_width
            };
            let next_mip_height = if mip_height > 1 {
                mip_height / 2
            } else {
                mip_height
            };

            barrier.subresource_range.base_mip_level = level - 1;
            barrier.old_layout = vk::ImageLayout::TRANSFER_DST_OPTIMAL;
            barrier.new_layout = vk::ImageLayout::TRANSFER_SRC_OPTIMAL;
            barrier.src_access_mask = vk::AccessFlags::TRANSFER_WRITE;
            barrier.dst_access_mask = vk::AccessFlags::TRANSFER_READ;
            let barriers = [barrier];

            unsafe {
                self.context.device().cmd_pipeline_barrier(
                    command_buffer,
                    vk::PipelineStageFlags::TRANSFER,
                    vk::PipelineStageFlags::TRANSFER,
                    vk::DependencyFlags::empty(),
                    &[],
                    &[],
                    &barriers,
                )
            };

            let blit = vk::ImageBlit::builder()
                .src_offsets([
                    vk::Offset3D { x: 0, y: 0, z: 0 },
                    vk::Offset3D {
                        x: mip_width,
                        y: mip_height,
                        z: 1,
                    },
                ])
                .src_subresource(vk::ImageSubresourceLayers {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    mip_level: level - 1,
                    base_array_layer: 0,
                    layer_count: self.layers,
                })
                .dst_offsets([
                    vk::Offset3D { x: 0, y: 0, z: 0 },
                    vk::Offset3D {
                        x: next_mip_width,
                        y: next_mip_height,
                        z: 1,
                    },
                ])
                .dst_subresource(vk::ImageSubresourceLayers {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    mip_level: level,
                    base_array_layer: 0,
                    layer_count: self.layers,
                })
                .build();
            let blits = [blit];

            unsafe {
                self.context.device().cmd_blit_image(
                    command_buffer,
                    self.image,
                    vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
                    self.image,
                    vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                    &blits,
                    vk::Filter::LINEAR,
                )
            };

            barrier.old_layout = vk::ImageLayout::TRANSFER_SRC_OPTIMAL;
            barrier.new_layout = vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL;
            barrier.src_access_mask = vk::AccessFlags::TRANSFER_READ;
            barrier.dst_access_mask = vk::AccessFlags::SHADER_READ;
            let barriers = [barrier];

            unsafe {
                self.context.device().cmd_pipeline_barrier(
                    command_buffer,
                    vk::PipelineStageFlags::TRANSFER,
                    vk::PipelineStageFlags::FRAGMENT_SHADER,
                    vk::DependencyFlags::empty(),
                    &[],
                    &[],
                    &barriers,
                )
            };

            mip_width = next_mip_width;
            mip_height = next_mip_height;
        }

        barrier.subresource_range.base_mip_level = self.mip_levels - 1;
        barrier.old_layout = vk::ImageLayout::TRANSFER_DST_OPTIMAL;
        barrier.new_layout = vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL;
        barrier.src_access_mask = vk::AccessFlags::TRANSFER_WRITE;
        barrier.dst_access_mask = vk::AccessFlags::SHADER_READ;
        let barriers = [barrier];

        unsafe {
            self.context.device().cmd_pipeline_barrier(
                command_buffer,
                vk::PipelineStageFlags::TRANSFER,
                vk::PipelineStageFlags::FRAGMENT_SHADER,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &barriers,
            )
        };
    }
}

impl Drop for ImageVariable {
    fn drop(&mut self) {
        unsafe {
            if !self.managed {
                self.context.device().destroy_image(self.image, None);
            }
            if let Some(memory) = self.memory {
                self.context.device().free_memory(memory, None);
            }
        }
    }
}

fn has_stencil_component(format: vk::Format) -> bool {
    format == vk::Format::D32_SFLOAT_S8_UINT || format == vk::Format::D24_UNORM_S8_UINT
}
