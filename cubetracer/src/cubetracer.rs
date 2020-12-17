use crate::camera::*;
use crate::context::Context;
use crate::datatypes::*;
use crate::mesh::Mesh;
use crate::pipeline::*;
use crate::window::*;

use world::ChunkMesh;

use nalgebra::{Vector2, Vector3};

use ash::version::DeviceV1_0;
use ash::vk;
use std::sync::Arc;

const SHADER_FOLDER: &str = "cubetracer/shaders";

pub struct Cubetracer {
    rtx_data: RTXData,
    camera: Camera,
}

impl Cubetracer {
    pub fn new(context: &Arc<Context>, swapchain: &Swapchain, ratio: f32, fov: f32) -> Self {
        let camera = Camera::new(
            Vector3::x(),
            Vector2::new(std::f32::consts::PI / 2.0, 0.0),
            fov,
            ratio,
        );

        let rtx_data = RTXData::new(context, swapchain, &camera);

        Cubetracer {
            rtx_data,
            camera,
        }
    }

    pub fn register_or_update_chunk(&mut self, chunk: ChunkMesh) {

    }

    pub fn delete_chunk(&mut self, x: i32, y: i32) {

    }

    pub fn camera(&mut self) -> &mut Camera {
        &mut self.camera
    }

    pub fn draw_frame(&mut self, context: &Arc<Context>) {
        self.rtx_data.uniform_scene.set(
            context,
            &UniformScene {
                sun_direction: self.camera.sun_direction(),
            },
        );
        self.rtx_data.uniform_camera.set(
            context, &self.camera.uniform(),
        );
    }

    pub fn commands(&self) -> &[vk::CommandBuffer] {
        self.rtx_data.get_command_buffers()
    }

    pub fn resize(&mut self, context: &Arc<Context>, swapchain: &Swapchain) {
        self.rtx_data = RTXData::new(context, swapchain, &self.camera);
    }
}

#[allow(dead_code)]
pub struct RTXData {
    context: Arc<Context>,

    output_texture: TextureVariable,
    uniform_camera: UniformVariable,
    uniform_scene: UniformVariable,
    vertices: BufferVariable,
    indices: BufferVariable,
    acceleration_structure: TlasVariable,

    command_buffers: Vec<vk::CommandBuffer>,

    pipeline: RaytracerPipeline,
}

impl RTXData {
    pub fn get_command_buffers(&self) -> &[vk::CommandBuffer] {
        &self.command_buffers
    }
}

impl RTXData {
    pub fn new(context: &Arc<Context>, swapchain: &Swapchain, camera: &Camera) -> Self {
        let mut output_texture = TextureVariable::from_swapchain(context, swapchain);
        let mut uniform_camera = UniformVariable::new(&context, &camera.uniform());
        let mut uniform_scene = UniformVariable::new(
            &context,
            &UniformScene {
                sun_direction: Vector3::new(0.5, 1.0, -0.5).normalize(),
            },
        );

        let (mut as_var, mut vertices, mut indices) = build_acceleration_structures(context);

        let pipeline = PipelineBuilder::new(context, SHADER_FOLDER)
            .binding(
                vk::DescriptorType::ACCELERATION_STRUCTURE_NV,
                &mut as_var,
                &[ShaderType::Raygen, ShaderType::ClosestHit],
            )
            .binding(
                vk::DescriptorType::STORAGE_IMAGE,
                &mut output_texture,
                &[ShaderType::Raygen],
            )
            .binding(
                vk::DescriptorType::UNIFORM_BUFFER,
                &mut uniform_camera,
                &[ShaderType::Raygen],
            )
            .binding(
                vk::DescriptorType::STORAGE_BUFFER,
                &mut vertices,
                &[ShaderType::ClosestHit],
            )
            .binding(
                vk::DescriptorType::STORAGE_BUFFER,
                &mut indices,
                &[ShaderType::ClosestHit],
            )
            .binding(
                vk::DescriptorType::UNIFORM_BUFFER,
                &mut uniform_scene,
                &[ShaderType::ClosestHit, ShaderType::Miss],
            )
            .real_shader(ShaderType::Raygen, "raygen.rgen.spv")
            .real_shader(ShaderType::Miss, "miss.rmiss.spv")
            .real_shader(ShaderType::Miss, "shadowmiss.rmiss.spv")
            .real_shader(ShaderType::ClosestHit, "closesthit.rchit.spv")
            .fake_shader(ShaderType::ClosestHit)
            .build(2);

        let mut rtx = Self {
            context: Arc::clone(context),

            // variables
            output_texture,
            uniform_camera,
            uniform_scene,
            vertices,
            indices,
            acceleration_structure: as_var,

            // pipeline
            pipeline,

            command_buffers: Vec::new(),
        };
        rtx.create_and_record_command_buffers(swapchain);

        rtx
    }

    pub fn update_acceleration_structure(&mut self, acceleration_structure: &mut TlasVariable) {
        self.pipeline.update_binding(0, acceleration_structure);
    }

    fn create_and_record_command_buffers(&mut self, swapchain: &Swapchain) {
        let device = self.context.device();
        let image_count = swapchain.image_count();

        {
            let allocate_info = vk::CommandBufferAllocateInfo::builder()
                .command_pool(self.context.general_command_pool())
                .level(vk::CommandBufferLevel::PRIMARY)
                .command_buffer_count(image_count as _);

            let buffers = unsafe {
                device
                    .allocate_command_buffers(&allocate_info)
                    .expect("Failed to allocate command buffers")
            };
            self.command_buffers.extend_from_slice(&buffers);
        };

        let command_buffer_begin_info = vk::CommandBufferBeginInfo::builder()
            .flags(vk::CommandBufferUsageFlags::SIMULTANEOUS_USE);

        self.command_buffers
            .iter()
            .enumerate()
            .for_each(|(index, buffer)| {
                let buffer = *buffer;
                let swapchain_image = &swapchain.images()[index];

                // begin command buffer
                unsafe {
                    device
                        .begin_command_buffer(buffer, &command_buffer_begin_info)
                        .expect("Failed to begin command buffer")
                };

                // Bind pipeline
                unsafe {
                    device.cmd_bind_pipeline(
                        buffer,
                        vk::PipelineBindPoint::RAY_TRACING_NV,
                        self.pipeline.pipeline,
                    )
                };

                // Bind descriptor set
                unsafe {
                    device.cmd_bind_descriptor_sets(
                        buffer,
                        vk::PipelineBindPoint::RAY_TRACING_NV,
                        self.pipeline.pipeline_layout,
                        0,
                        &self.pipeline.descriptor_sets,
                        &[],
                    );
                };

                let swapchain_props = swapchain.properties();

                // Trace rays
                let shader_group_handle_size = self.pipeline.rt_properties.shader_group_handle_size;
                let raygen_offset = 0;
                let miss_offset = shader_group_handle_size;
                let hit_offset = 3 * shader_group_handle_size;

                unsafe {
                    let sbt_buffer = *self.pipeline.shader_binding_table_buffer.buffer();
                    self.context.ray_tracing().cmd_trace_rays(
                        buffer,
                        sbt_buffer,
                        raygen_offset,
                        sbt_buffer,
                        miss_offset.into(),
                        shader_group_handle_size.into(),
                        sbt_buffer,
                        hit_offset.into(),
                        shader_group_handle_size.into(),
                        vk::Buffer::null(),
                        0,
                        0,
                        swapchain_props.extent.width,
                        swapchain_props.extent.height,
                        1,
                    );
                };

                // Copy output image to swapchain
                {
                    // transition layouts
                    swapchain_image.cmd_transition_image_layout(
                        buffer,
                        vk::ImageLayout::UNDEFINED,
                        vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                    );
                    self.output_texture.image.cmd_transition_image_layout(
                        buffer,
                        vk::ImageLayout::GENERAL,
                        vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
                    );

                    // Copy image
                    let image_copy_info = [vk::ImageCopy::builder()
                        .src_subresource(vk::ImageSubresourceLayers {
                            aspect_mask: vk::ImageAspectFlags::COLOR,
                            mip_level: 0,
                            base_array_layer: 0,
                            layer_count: 1,
                        })
                        .src_offset(vk::Offset3D { x: 0, y: 0, z: 0 })
                        .dst_subresource(vk::ImageSubresourceLayers {
                            aspect_mask: vk::ImageAspectFlags::COLOR,
                            mip_level: 0,
                            base_array_layer: 0,
                            layer_count: 1,
                        })
                        .dst_offset(vk::Offset3D { x: 0, y: 0, z: 0 })
                        .extent(vk::Extent3D {
                            width: swapchain_props.extent.width,
                            height: swapchain_props.extent.height,
                            depth: 1,
                        })
                        .build()];

                    unsafe {
                        device.cmd_copy_image(
                            buffer,
                            self.output_texture.image.image,
                            vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
                            swapchain_image.image,
                            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                            &image_copy_info,
                        );
                    };

                    // Transition layout
                    swapchain_image.cmd_transition_image_layout(
                        buffer,
                        vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                        vk::ImageLayout::PRESENT_SRC_KHR,
                    );
                    self.output_texture.image.cmd_transition_image_layout(
                        buffer,
                        vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
                        vk::ImageLayout::GENERAL,
                    );
                }

                // End command buffer
                unsafe {
                    device
                        .end_command_buffer(buffer)
                        .expect("Failed to end command buffer")
                };
            });
    }
}

fn build_acceleration_structures(
    context: &Arc<Context>,
) -> (TlasVariable, BufferVariable, BufferVariable) {
    let mesh = Mesh::from_file("assets/models/dog/dog.obj");

    let vertices = mesh.device_vertices(context);
    let indices = mesh.device_indices(context);

    let blas = BlasVariable::from_geometry(context, &vertices, &indices);
    let mut tlas = TlasVariable::from_blas_list(context, vec![blas]);
    tlas.build(context);

    (
        tlas,
        vertices,
        indices,
    )
}

impl Drop for RTXData {
    fn drop(&mut self) {
        let device = self.context.device();
        unsafe {
            device.free_command_buffers(self.context.general_command_pool(), &self.command_buffers);
            device.destroy_pipeline(self.pipeline.pipeline, None);
            device.destroy_pipeline_layout(self.pipeline.pipeline_layout, None);
            device.destroy_descriptor_pool(self.pipeline.descriptor_pool, None);
            device.destroy_descriptor_set_layout(self.pipeline.descriptor_set_layout, None);
        }
    }
}
