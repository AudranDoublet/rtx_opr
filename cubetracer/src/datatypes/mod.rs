use ash::vk;

mod acceleration_structure;
mod buffer;
mod image;
mod texture;
mod uniforms;

pub trait DataType {
    fn write_descriptor_builder(&mut self) -> vk::WriteDescriptorSetBuilder;
}

pub use acceleration_structure::*;
pub use buffer::*;
pub use image::*;
pub use texture::*;
pub use uniforms::*;
