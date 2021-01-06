use ash::version::DeviceV1_0;
use ash::vk;

use crate::context::Context;
use crate::datatypes::*;

use std::collections::BTreeMap;
use std::sync::Arc;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct InstanceBinding {
    pub indices: vk::Buffer,
    pub triangles: vk::Buffer,
}

pub struct TlasVariable {
    modified: bool,

    pub blas_map: BTreeMap<BlasName, BlasVariable>,

    instance_buffer: Option<BufferVariable>,

    pub acceleration_structure: Option<AccelerationStructure>,
    structures: Vec<vk::AccelerationStructureNV>,
    info: Option<vk::WriteDescriptorSetAccelerationStructureNV>,
}

impl TlasVariable {
    pub fn new() -> TlasVariable {
        TlasVariable {
            blas_map: BTreeMap::new(),
            structures: vec![],
            acceleration_structure: None,
            instance_buffer: None,
            info: None,
            modified: true,
        }
    }

    pub fn register(&mut self, name: BlasName, blas: BlasVariable) {
        self.blas_map.insert(name, blas);
        self.modified = true;
    }

    pub fn unregister(&mut self, name: BlasName) {
        if self.blas_map.contains_key(&name) {
            self.blas_map.remove(&name);
            self.modified = true;
        }
    }

    pub fn get_blas_textures(&self) -> BufferVariableList {
        BufferVariableList::new(
            self.blas_map
                .values()
                .into_iter()
                .map(|blas| *blas.textures().buffer())
                .collect(),
        )
    }

    pub fn get_blas_data(&self) -> BufferVariableList {
        BufferVariableList::new(
            self.blas_map
                .values()
                .into_iter()
                .map(|blas| *blas.triangle_data().buffer())
                .collect(),
        )
    }

    /// build or rebuild the acceleration structure
    pub fn build(&mut self, context: &Arc<Context>, bindings: &mut [InstanceBinding]) -> bool {
        if !self.modified {
            return false;
        }

        self.modified = false;

        let data = self
            .blas_map
            .iter()
            .map(|(_, v)| v.instance_data())
            .collect::<Vec<_>>();

        self.blas_map.iter().enumerate().for_each(|(i, (_, v))| {
            bindings[i] = v.bindings();
        });

        if let Some(acceleration_structure) = self.acceleration_structure.as_mut() {
            acceleration_structure
                .acceleration_structure_info
                .instance_count = data.len() as u32;
        } else {
            let acceleration_structure_info = vk::AccelerationStructureInfoNV::builder()
                .ty(vk::AccelerationStructureTypeNV::TOP_LEVEL)
                .instance_count(data.len() as u32)
                .build();

            self.acceleration_structure = Some(AccelerationStructure::new(
                Arc::clone(context),
                acceleration_structure_info,
            ));

            self.structures = vec![
                self.acceleration_structure
                    .as_ref()
                    .unwrap()
                    .acceleration_structure,
            ];
        }

        // Build instance buffer (list & informations of each BLAS)
        let instance_buffer = if data.len() == 0 {
            BufferVariable::null("null_instance_buffer_tlas".to_string(), context)
        } else {
            BufferVariable::device_buffer(
                "instance_buffer_tlas".to_string(),
                context,
                vk::BufferUsageFlags::RAY_TRACING_NV,
                &data,
            )
            .0
        };

        // create scratch buffer === size is maximum size needed to build the TLAS or one of the BLAS
        let scratch_buffer_size = self
            .blas_map
            .iter()
            .filter_map(|(_, v)| v.build_memory_requirements())
            .max()
            .unwrap_or(0)
            .max(
                self.acceleration_structure
                    .as_ref()
                    .unwrap()
                    .get_memory_requirements(
                        vk::AccelerationStructureMemoryRequirementsTypeNV::BUILD_SCRATCH,
                    )
                    .memory_requirements
                    .size,
            );

        let scratch_buffer = BufferVariable::create(
            "scratch_buffer_tlas".to_string(),
            context,
            scratch_buffer_size,
            scratch_buffer_size as _,
            vk::BufferUsageFlags::RAY_TRACING_NV,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
        );

        // build
        context.execute_one_time_commands(|command_buffer| {
            let memory_barrier = [vk::MemoryBarrier::builder()
                .src_access_mask(
                    vk::AccessFlags::ACCELERATION_STRUCTURE_READ_NV
                        | vk::AccessFlags::ACCELERATION_STRUCTURE_WRITE_NV,
                )
                .dst_access_mask(
                    vk::AccessFlags::ACCELERATION_STRUCTURE_READ_NV
                        | vk::AccessFlags::ACCELERATION_STRUCTURE_WRITE_NV,
                )
                .build()];

            // Build bottom AS
            self.blas_map.iter_mut().for_each(|(_, blas)| {
                if blas.build(command_buffer, &scratch_buffer) {
                    // memory barrier if we build the BLAS
                    unsafe {
                        context.device().cmd_pipeline_barrier(
                            command_buffer,
                            vk::PipelineStageFlags::ACCELERATION_STRUCTURE_BUILD_NV,
                            vk::PipelineStageFlags::ACCELERATION_STRUCTURE_BUILD_NV,
                            vk::DependencyFlags::empty(),
                            &memory_barrier,
                            &[],
                            &[],
                        )
                    };
                }
            });

            // Build top AS
            self.acceleration_structure.as_ref().unwrap().cmd_build(
                command_buffer,
                &scratch_buffer,
                Some(&instance_buffer),
            );

            unsafe {
                context.device().cmd_pipeline_barrier(
                    command_buffer,
                    vk::PipelineStageFlags::ACCELERATION_STRUCTURE_BUILD_NV,
                    vk::PipelineStageFlags::ACCELERATION_STRUCTURE_BUILD_NV,
                    vk::DependencyFlags::empty(),
                    &memory_barrier,
                    &[],
                    &[],
                )
            };
        });

        self.instance_buffer = Some(instance_buffer);
        true
    }
}

impl DataType for TlasVariable {
    fn write_descriptor_builder(&mut self) -> vk::WriteDescriptorSetBuilder {
        self.info = Some(
            vk::WriteDescriptorSetAccelerationStructureNV::builder()
                .acceleration_structures(&self.structures)
                .build(),
        );

        vk::WriteDescriptorSet::builder().push_next(self.info.as_mut().unwrap())
    }

    fn len(&self) -> usize {
        1
    }
}
