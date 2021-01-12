use crate::barriers::image_barrier;
use crate::camera::*;
use crate::context::Context;
use crate::datatypes::*;
use crate::descriptors::*;
use crate::pipeline::*;
use crate::window::*;

use world::{main_world, ChunkMesh};

use nalgebra::{Vector2, Vector3};

use ash::vk;
use std::collections::HashMap;
use std::sync::Arc;

const SHADER_FOLDER: &str = "cubetracer/shaders";
const MAX_INSTANCE_BINDING: usize = 1024;

pub struct Cubetracer {
    chunks: HashMap<BlasName, ChunkMesh>,
    camera: Camera,

    rtx_data: Option<RTXData>,

    acceleration_structure: Option<TlasVariable>,
    texture_array: TextureVariable,
    uniform_scene: UniformVariable,
    uniform_camera: UniformVariable,

    local_instance_bindings: [InstanceBinding; MAX_INSTANCE_BINDING],
}

impl Cubetracer {
    pub fn new(context: &Arc<Context>, ratio: f32, fov: f32) -> Self {
        let camera = Camera::new(
            Vector3::x(),
            Vector2::new(std::f32::consts::PI / 2.0, 0.0),
            fov,
            ratio,
        );

        let textures_info = &main_world().textures;
        let (w, h) = textures_info.dimensions();

        let texture_array =
            TextureVariable::texture_array2d(context, w as u32, h as u32, textures_info.paths());

        let local_instance_bindings = [InstanceBinding {
            indices: vk::Buffer::null(),
            triangles: vk::Buffer::null(),
        }; MAX_INSTANCE_BINDING];

        Cubetracer {
            chunks: HashMap::new(),
            rtx_data: None,
            acceleration_structure: None,
            local_instance_bindings,
            texture_array,
            uniform_scene: UniformVariable::new(
                &context,
                &UniformScene {
                    sun_direction: Vector3::new(0.5, 1.0, -0.5).normalize(),
                },
            ),
            uniform_camera: UniformVariable::new(&context, &camera.uniform()),
            camera,
        }
    }

    fn begin(
        &mut self,
        context: &Arc<Context>,
        swapchain: &Swapchain,
        name: BlasName,
        blas: BlasVariable,
    ) {
        let mut acceleration_structure = TlasVariable::new();
        acceleration_structure.register(name, blas);
        acceleration_structure.build(context, &mut self.local_instance_bindings);
        self.acceleration_structure = Some(acceleration_structure);

        let rtx_data = RTXData::new(context, swapchain, self);

        self.rtx_data = Some(rtx_data);
    }

    pub fn register_or_update_chunk(
        &mut self,
        context: &Arc<Context>,
        swapchain: &Swapchain,
        x: i32,
        y: i32,
        chunk: ChunkMesh,
    ) {
        let name = BlasName::Chunk(x, y);
        self.chunks.insert(name, chunk);

        let chunk = &self.chunks[&name];
        let vertices = BufferVariable::device_buffer(
            "blas_vertices".to_string(),
            context,
            vk::BufferUsageFlags::VERTEX_BUFFER | vk::BufferUsageFlags::STORAGE_BUFFER,
            &chunk.vertices,
        )
        .0;

        let indices = BufferVariable::device_buffer(
            "blas_indices".to_string(),
            context,
            vk::BufferUsageFlags::VERTEX_BUFFER | vk::BufferUsageFlags::STORAGE_BUFFER,
            &chunk.indices,
        )
        .0;

        let triangle_data = BufferVariable::device_buffer(
            "blas_triangles_data".to_string(),
            context,
            vk::BufferUsageFlags::STORAGE_BUFFER,
            &chunk.triangle_data,
        )
        .0;

        let textures = BufferVariable::device_buffer(
            "blas_textures".to_string(),
            context,
            vk::BufferUsageFlags::STORAGE_BUFFER,
            &chunk.texture_vertices,
        )
        .0;

        let blas = BlasVariable::from_geometry(
            context,
            vertices,
            indices,
            triangle_data,
            textures,
            std::mem::size_of::<[f32; 4]>(),
        );

        if let Some(acceleration_structure) = self.acceleration_structure.as_mut() {
            acceleration_structure.register(name, blas);
        } else {
            self.begin(context, swapchain, name, blas);
        }
    }

    pub fn delete_chunk(&mut self, x: i32, y: i32) {
        let name = BlasName::Chunk(x, y);

        if self.chunks.contains_key(&name) {
            self.chunks.remove(&name);
        }
        if let Some(acceleration_structure) = self.acceleration_structure.as_mut() {
            acceleration_structure.unregister(name);
        }
    }

    pub fn camera(&mut self) -> &mut Camera {
        &mut self.camera
    }

    pub fn update(&mut self, context: &Arc<Context>) -> bool {
        if let Some(acceleration_structure) = self.acceleration_structure.as_mut() {
            if acceleration_structure.build(context, &mut self.local_instance_bindings) {
                self.rtx_data.as_mut().unwrap().update_blas_data(
                    &mut acceleration_structure.get_blas_data(),
                    &mut acceleration_structure.get_blas_textures(),
                );
            }

            self.uniform_scene.set(
                context,
                &UniformScene {
                    sun_direction: self.camera.sun_direction(),
                },
            );

            self.uniform_camera.set(context, &self.camera.uniform());

            true
        } else {
            false
        }
    }

    pub fn commands(&self) -> &[vk::CommandBuffer] {
        self.rtx_data.as_ref().unwrap().get_command_buffers()
    }

    pub fn resize(&mut self, context: &Arc<Context>, swapchain: &Swapchain) {
        self.rtx_data = Some(RTXData::new(context, swapchain, self));

        if let Some(acceleration_structure) = self.acceleration_structure.as_mut() {
            self.rtx_data.as_mut().unwrap().update_blas_data(
                &mut acceleration_structure.get_blas_data(),
                &mut acceleration_structure.get_blas_textures(),
            );
        }
    }
}

#[allow(dead_code)]
pub struct RTXData {
    context: Arc<Context>,

    output_texture: TextureVariable,
    cache: Vec<TextureVariable>,

    command_buffers: CommandBuffers,

    descriptor_sets: Vec<DescriptorSet>,

    pipeline: RaytracerPipeline,
    reconstruct_pipeline: ComputePipeline,
}

impl RTXData {
    pub fn get_command_buffers(&self) -> &[vk::CommandBuffer] {
        &self.command_buffers.buffers()
    }

    pub fn update_blas_data(
        &mut self,
        data: &mut BufferVariableList,
        textures: &mut BufferVariableList,
    ) {
        self.descriptor_sets[0]
            .update(&self.context)
            .register(4, vk::DescriptorType::STORAGE_BUFFER, data)
            .register(5, vk::DescriptorType::STORAGE_BUFFER, textures)
            .update();
    }
}

impl RTXData {
    pub fn new(context: &Arc<Context>, swapchain: &Swapchain, cubetracer: &mut Cubetracer) -> Self {
        ////// CREATE CACHES
        let mut output_texture = TextureVariable::from_swapchain(context, swapchain);

        let mut cache_normals = TextureVariable::from_swapchain_format(
            context,
            swapchain,
            vk::Format::R32G32B32A32_SFLOAT,
        );
        let mut cache_initial_distances =
            TextureVariable::from_swapchain_format(context, swapchain, vk::Format::R32_SFLOAT);
        let mut cache_direct_illuminations = TextureVariable::from_swapchain_format(
            context,
            swapchain,
            vk::Format::R32G32B32A32_SFLOAT,
        );
        let mut cache_origin = TextureVariable::from_swapchain_format(
            context,
            swapchain,
            vk::Format::R32G32B32A32_SFLOAT,
        );
        let mut cache_shadows = TextureVariable::from_swapchain_format(
            context,
            swapchain,
            vk::Format::R32G32B32A32_SFLOAT,
        );
        let mut cache_mer = TextureVariable::from_swapchain_format(
            context,
            swapchain,
            vk::Format::R32G32B32A32_SFLOAT,
        );
        let mut cache_pt_origins = TextureVariable::from_swapchain_format(
            context,
            swapchain,
            vk::Format::R32G32B32A32_SFLOAT,
        );
        let mut cache_pt_normals = TextureVariable::from_swapchain_format(
            context,
            swapchain,
            vk::Format::R32G32B32A32_SFLOAT,
        );
        let mut cache_pt_illum = TextureVariable::from_swapchain_format(
            context,
            swapchain,
            vk::Format::R32G32B32A32_SFLOAT,
        );

        let max_nb_chunks = MAX_INSTANCE_BINDING; // FIXME: replace with the real max number of visible chunks

        ////// CREATE DESCRIPTOR SETS
        let descriptor_set = DescriptorSetBuilder::new(context)
            .binding(
                // 0
                vk::DescriptorType::ACCELERATION_STRUCTURE_NV,
                cubetracer.acceleration_structure.as_mut().unwrap(),
                &[ShaderType::Raygen, ShaderType::ClosestHit],
            )
            .binding(
                // 1
                vk::DescriptorType::UNIFORM_BUFFER,
                &mut cubetracer.uniform_camera,
                &[ShaderType::Raygen, ShaderType::Compute],
            )
            .binding(
                // 2
                vk::DescriptorType::UNIFORM_BUFFER,
                &mut cubetracer.uniform_scene,
                &[ShaderType::Raygen, ShaderType::ClosestHit, ShaderType::Miss],
            )
            .binding(
                // 3
                vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                &mut cubetracer.texture_array,
                &[ShaderType::ClosestHit, ShaderType::AnyHit],
            )
            .binding_count(
                // 4
                vk::DescriptorType::STORAGE_BUFFER,
                max_nb_chunks as u32,
                &mut BufferVariableList::empty(max_nb_chunks),
                &[ShaderType::ClosestHit, ShaderType::AnyHit],
            )
            .binding_count(
                // 5
                vk::DescriptorType::STORAGE_BUFFER,
                max_nb_chunks as u32,
                &mut BufferVariableList::empty(max_nb_chunks),
                &[ShaderType::ClosestHit, ShaderType::AnyHit],
            )
            .build();

        let cache_descriptors = DescriptorSetBuilder::new(context)
            .bindings(
                // 0 - 7
                vk::DescriptorType::STORAGE_IMAGE,
                vec![
                    &mut output_texture,
                    &mut cache_normals,
                    &mut cache_initial_distances,
                    &mut cache_direct_illuminations,
                    &mut cache_origin,
                    &mut cache_shadows,
                    &mut cache_mer,
                    &mut cache_pt_origins,
                    &mut cache_pt_normals,
                    &mut cache_pt_illum,
                ],
                &[ShaderType::Raygen, ShaderType::Compute],
            )
            .build();
        ////// CREATE PIPELINES
        let pipeline = PipelineBuilder::new(context, SHADER_FOLDER)
            .general_shader(ShaderType::Raygen, "initial/raygen.rgen.spv")
            .general_shader(ShaderType::Raygen, "shadow/raygen.rgen.spv")
            .general_shader(ShaderType::Raygen, "path_tracing/raygen.rgen.spv")

            .general_shader(ShaderType::Miss, "initial/miss.rmiss.spv")
            .general_shader(ShaderType::Miss, "shadow/miss.rmiss.spv")
            .general_shader(ShaderType::Miss, "path_tracing/miss.rmiss.spv")

            .hit_shaders(
                Some("initial/closesthit.rchit.spv"),
                Some("initial/anyhit.rahit.spv"),
            )
            .hit_shaders(None, Some("shadow/anyhit.rahit.spv"))
            .descriptor_set(&descriptor_set)
            .descriptor_set(&cache_descriptors)
            .build(3);

        let reconstruct_pipeline = ComputePipelineBuilder::new(context, SHADER_FOLDER)
            .shader("reconstruct.comp.spv")
            .descriptor_set(&cache_descriptors)
            .build();

        ////// CREATE COMMANDS
        let command_buffers = CommandBuffers::new(context, swapchain);
        command_buffers.record(|index, buffer| {
            let swapchain_props = swapchain.properties();

            let width = swapchain_props.extent.width;
            let height = swapchain_props.extent.height;

            // Initial ray
            pipeline.bind(&context, buffer);
            pipeline.dispatch(buffer, width, height, 0);

            image_barrier(
                &context,
                buffer,
                &[
                    // general buffers
                    cache_normals.image.image,
                    cache_initial_distances.image.image,
                    cache_origin.image.image,
                    cache_mer.image.image,
                    cache_direct_illuminations.image.image,

                    // path tracing buffers, FIXME: should just init them with a copy
                    cache_pt_normals.image.image,
                    cache_pt_origins.image.image,
                    cache_pt_illum.image.image,
                ],
            );

            // Path tracing with _ bouncing rays
            for _ in 0..1 {
                pipeline.dispatch(buffer, width, height, 2);

                image_barrier(
                    &context,
                    buffer,
                    &[
                        cache_pt_normals.image.image,
                        cache_pt_origins.image.image,
                        cache_pt_illum.image.image,
                    ],
                );
            }

            // Shadows
            pipeline.dispatch(buffer, width, height, 1);

            image_barrier(
                &context,
                buffer,
                &[
                    cache_shadows.image.image,
                ],
            );

            // Reconstruct
            reconstruct_pipeline.bind(&context, buffer);
            reconstruct_pipeline.dispatch(buffer, width, height);

            image_barrier(context, buffer, &[output_texture.image.image]);

            swapchain.cmd_update_image(buffer, index, &output_texture.image);
        });

        Self {
            context: Arc::clone(context),

            // variables
            output_texture,

            cache: vec![
                cache_normals,
                cache_initial_distances,
                cache_direct_illuminations,
                cache_origin,
                cache_shadows,
                cache_mer,

                cache_pt_origins,
                cache_pt_normals,
                cache_pt_illum
            ],
            descriptor_sets: vec![descriptor_set, cache_descriptors],

            pipeline,
            reconstruct_pipeline,

            command_buffers,
        }
    }
}
