use ash::vk;
use ash::version::DeviceV1_0;

use crate::context::Context;
use crate::datatypes::*;

use std::sync::Arc;

pub enum BlasName {
    Chunk(i32, i32),
}

pub struct TlasVariable {
    blas: Vec<BlasVariable>,
    instance_buffer: Option<BufferVariable>,
    acceleration_structure: AccelerationStructure,

    structures: Vec<vk::AccelerationStructureNV>,
    info: Option<vk::WriteDescriptorSetAccelerationStructureNV>,
}

impl TlasVariable {
    pub fn from_blas_list(
        context: &Arc<Context>,
        blas: Vec<BlasVariable>,
    ) -> TlasVariable {
        let acceleration_structure_info = vk::AccelerationStructureInfoNV::builder()
            .ty(vk::AccelerationStructureTypeNV::TOP_LEVEL)
            .instance_count(blas.len() as u32)
            .build();
        let acceleration_structure = AccelerationStructure::new(Arc::clone(context), acceleration_structure_info);

        TlasVariable {
            structures: vec![acceleration_structure.acceleration_structure],
            acceleration_structure,
            blas,
            instance_buffer: None,
            info: None,
        }
    }

    pub fn acceleration_structure(&self) -> vk::AccelerationStructureNV {
        self.acceleration_structure.acceleration_structure
    }

    pub fn build(&mut self, context: &Arc<Context>) {
        let data = self.blas.iter().map(|v| v.instance_data()).collect::<Vec<_>>();

        let instance_buffer = BufferVariable::device_buffer(
            context,
            vk::BufferUsageFlags::RAY_TRACING_NV,
            &data,
        ).0;

        // Build acceleration structure
        let scratch_buffer_size = self.blas
            .iter().filter_map(|v| v.build_memory_requirements())
            .max()
            .unwrap_or(0)
            .max(self.acceleration_structure.get_memory_requirements(
                vk::AccelerationStructureMemoryRequirementsTypeNV::BUILD_SCRATCH,
            ).memory_requirements.size);

        let scratch_buffer = BufferVariable::create(
            context,
            scratch_buffer_size,
            scratch_buffer_size as _,
            vk::BufferUsageFlags::RAY_TRACING_NV,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
        );

        context.execute_one_time_commands(|command_buffer| {
            let memory_barrier = [
                vk::MemoryBarrier::builder()
                .src_access_mask(
                    vk::AccessFlags::ACCELERATION_STRUCTURE_READ_NV
                        | vk::AccessFlags::ACCELERATION_STRUCTURE_WRITE_NV,
                )
                .dst_access_mask(
                    vk::AccessFlags::ACCELERATION_STRUCTURE_READ_NV
                        | vk::AccessFlags::ACCELERATION_STRUCTURE_WRITE_NV,
                )
                .build()
            ];

            // Build bottom AS
            self.blas.iter_mut().for_each(|blas| {
                blas.build(command_buffer, &scratch_buffer);

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

            // Build top AS
            self.acceleration_structure.cmd_build(
                command_buffer, &scratch_buffer, Some(&instance_buffer)
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
    }
}

impl DataType for TlasVariable {
    fn write_descriptor_builder(&mut self) -> vk::WriteDescriptorSetBuilder {

        self.info = Some(
            vk::WriteDescriptorSetAccelerationStructureNV::builder()
                .acceleration_structures(&self.structures)
                .build(),
        );

        match self.info {
            Some(ref mut e) => vk::WriteDescriptorSet::builder().push_next(e),
            None => panic!("should not happen"),
        }
    }
}
