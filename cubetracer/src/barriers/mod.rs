
use ash::vk;
use ash::version::DeviceV1_0;
use crate::context::Context;


fn _make_im_mem_barriers(images: &[vk::Image]) -> Vec<vk::ImageMemoryBarrier> {
    images.iter().map(|&img| 
        vk::ImageMemoryBarrier{
            s_type: vk::StructureType::IMAGE_MEMORY_BARRIER,
            src_access_mask: vk::AccessFlags::SHADER_WRITE,
            dst_access_mask: vk::AccessFlags::SHADER_READ,
            new_layout: vk::ImageLayout::GENERAL,
            old_layout: vk::ImageLayout::GENERAL,
            image: img,
            ..Default::default() 
        }
    ).collect()
}

pub fn image_barrier(context: &Context, buffer: vk::CommandBuffer, images: &[vk::Image]) {
    let im_mem_barrier = _make_im_mem_barriers(images);

    unsafe{
        context.device().cmd_pipeline_barrier(
            buffer,
            vk::PipelineStageFlags::ALL_COMMANDS, // src stage
            vk::PipelineStageFlags::ALL_COMMANDS, // dst stage
            vk::DependencyFlags::empty(), // dependency_flags
            &[], // memory_barrier
            &[], // buffer_memory_barriers
            &im_mem_barrier, // image_memory_barriers
        );
    }
}
