use crate::barriers::image_barrier;
use crate::camera::*;
use crate::context::Context;
use crate::datatypes::*;
use crate::descriptors::*;
use crate::pipeline::*;
use crate::window::*;

use crate::cache_buffers::*;

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

    acceleration_structure: TlasVariable,
    texture_array: TextureVariable,
    uniform_scene: UniformVariable,
    uniform_camera: UniformVariable,

    local_instance_bindings: [InstanceBinding; MAX_INSTANCE_BINDING],
    rendered_buffer: u32,
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
            acceleration_structure: TlasVariable::new(),
            local_instance_bindings,
            texture_array,
            uniform_scene: UniformVariable::new(
                &context,
                &UniformScene {
                    sun_direction: Vector3::new(0.5, 1.0, -0.5).normalize(),
                    rendered_buffer: 0,
                },
            ),
            uniform_camera: UniformVariable::new(&context, &camera.uniform()),
            camera,
            rendered_buffer: 0,
        }
    }

    pub fn set_rendered_buffer(&mut self, buffer: u32) {
        self.rendered_buffer = buffer;
    }

    pub fn register_or_update_chunk(
        &mut self,
        context: &Arc<Context>,
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

        self.acceleration_structure.register(name, blas);
    }

    pub fn delete_chunk(&mut self, x: i32, y: i32) {
        let name = BlasName::Chunk(x, y);

        if self.chunks.contains_key(&name) {
            self.chunks.remove(&name);
        }

        self.acceleration_structure.unregister(name);
    }

    pub fn camera(&mut self) -> &mut Camera {
        &mut self.camera
    }

    pub fn update(&mut self, swapchain: &Swapchain, context: &Arc<Context>) -> bool {
        if self.chunks.len() >= 5 {
            if self.rtx_data.is_none() {
                self.acceleration_structure
                    .build(context, &mut self.local_instance_bindings);

                let rtx_data = RTXData::new(context, swapchain, self);
                self.rtx_data = Some(rtx_data);
            }

            let rtx_data = self.rtx_data.as_mut().unwrap();

            rtx_data.update_cache_buffers();

            if self.acceleration_structure.build(context, &mut self.local_instance_bindings) {
                rtx_data.update_blas_data(
                    &mut self.acceleration_structure.get_blas_data(),
                    &mut self.acceleration_structure.get_blas_textures(),
                );
            }

            self.uniform_scene.set(
                context,
                &UniformScene {
                    sun_direction: self.camera.sun_direction(),
                    rendered_buffer: self.rendered_buffer,
                },
            );

            self.uniform_camera.set(context, &self.camera.uniform());
            self.camera.store_previous_view();

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

        self.rtx_data.as_mut().unwrap().update_blas_data(
            &mut self.acceleration_structure.get_blas_data(),
            &mut self.acceleration_structure.get_blas_textures(),
        );
    }
}

#[allow(dead_code)]
pub struct RTXData {
    context: Arc<Context>,

    cache_buffers: BufferList,

    command_buffers: CommandBuffers,

    descriptor_sets: Vec<DescriptorSet>,

    pipeline: RaytracerPipeline,
    pipeline_reflection: RaytracerPipeline,
    reconstruct_pipeline: ComputePipeline,
}

impl RTXData {
    pub fn get_command_buffers(&self) -> &[vk::CommandBuffer] {
        &self.command_buffers.buffers()
    }

    pub fn update_cache_buffers(&mut self) {
        self.cache_buffers.update(&self.descriptor_sets[1]);
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
        let mut cache_buffers = BufferList::new(context);

        cache_buffers
            .simple_same("output_texture", swapchain)
            .double("history_length", swapchain, BufferFormat::VALUE)
            .double("moments", swapchain, BufferFormat::RGBA)
            .simple("normals", swapchain, BufferFormat::RGBA)
            .double("initial_distances", swapchain, BufferFormat::VALUE)
            .double("direct_illumination", swapchain, BufferFormat::RGBA)
            .simple("hit_point", swapchain, BufferFormat::RGBA)
            .simple("shadow", swapchain, BufferFormat::RGBA)
            .simple("mer", swapchain, BufferFormat::RGBA)
            .simple("pathtracing_origin", swapchain, BufferFormat::RGBA)
            .simple("pathtracing_normal", swapchain, BufferFormat::RGBA)
            .simple("pathtracing_illum", swapchain, BufferFormat::RGBA)
        ;

        let max_nb_chunks = MAX_INSTANCE_BINDING; // FIXME: replace with the real max number of visible chunks

        ////// CREATE DESCRIPTOR SETS
        let descriptor_set = DescriptorSetBuilder::new(context)
            .binding(
                // 0
                vk::DescriptorType::ACCELERATION_STRUCTURE_NV,
                &mut cubetracer.acceleration_structure,
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
                &[ShaderType::Raygen, ShaderType::ClosestHit, ShaderType::Miss, ShaderType::Compute],
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

        let cache_descriptors = cache_buffers.descriptor_set(&[ShaderType::Raygen, ShaderType::Compute]);

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
            .hit_shaders(None, Some("initial/anyhit.rahit.spv"))
            .descriptor_set(&descriptor_set)
            .descriptor_set(&cache_descriptors)
            .build(3);

        let pipeline_reflection = PipelineBuilder::new(context, SHADER_FOLDER)
            .general_shader(ShaderType::Raygen, "path_tracing/raygen.rgen.spv")
            .general_shader(ShaderType::Miss, "path_tracing/miss.rmiss.spv")
            .general_shader(ShaderType::Miss, "shadow/miss.rmiss.spv")

            .hit_shaders(
                Some("initial/closesthit.rchit.spv"),
                Some("initial/anyhit.rahit.spv"),
            )
            .hit_shaders(None, Some("initial/anyhit.rahit.spv"))

            .descriptor_set(&descriptor_set)
            .descriptor_set(&cache_descriptors)
            .build(3);

        let reconstruct_pipeline = ComputePipelineBuilder::new(context, SHADER_FOLDER)
            .shader("reconstruct.comp.spv")
            .descriptor_set(&descriptor_set) //FIXME only uniforms
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
                &cache_buffers.images(&[
                    "normals",
                    "initial_distances",
                    "hit_point",
                    "mer",
                    "direct_illumination",
                    "pathtracing_origin",
                    "pathtracing_normal",
                    "pathtracing_illum",
                ])
            );

            // Shadows
            pipeline.dispatch(buffer, width, height, 1);

            // Path tracing with _ bouncing rays
            for _ in 0..1 {
                pipeline.dispatch(buffer, width, height, 2);

                image_barrier(
                    &context,
                    buffer,
                    &cache_buffers.images(&[
                        "pathtracing_origin",
                        "pathtracing_normal",
                        "pathtracing_illum",
                    ]),
                );
            }
            image_barrier(&context, buffer, &cache_buffers.images(&["shadow"]));

            // Reconstruct
            reconstruct_pipeline.bind(&context, buffer);
            reconstruct_pipeline.dispatch(buffer, width, height);

            image_barrier(&context, buffer, &cache_buffers.images(&["output_texture"]));

            swapchain.cmd_update_image(buffer, index, &cache_buffers.texture("output_texture").image);
        });

        Self {
            context: Arc::clone(context),

            // variables
            cache_buffers,
            descriptor_sets: vec![descriptor_set, cache_descriptors],

            pipeline,
            pipeline_reflection,
            reconstruct_pipeline,

            command_buffers,
        }
    }
}
