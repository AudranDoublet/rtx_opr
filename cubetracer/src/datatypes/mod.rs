use ash::vk;

mod acceleration_structure;
mod buffer;
mod image;
mod texture;
mod uniforms;
mod blas;
mod tlas;
mod buffer_list;

pub trait DataType {
    fn write_descriptor_builder(&mut self) -> vk::WriteDescriptorSetBuilder;
    fn len(&self) -> usize;
}

pub use acceleration_structure::*;
pub use blas::*;
pub use tlas::*;
pub use buffer::*;
pub use self::image::*;
pub use texture::*;
pub use uniforms::*;
pub use buffer_list::*;
