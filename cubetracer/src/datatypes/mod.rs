use ash::vk;

mod acceleration_structure;
mod blas;
mod buffer;
mod buffer_list;
mod image;
mod texture;
mod tlas;
mod uniforms;

pub trait DataType {
    fn write_descriptor_builder(&mut self) -> vk::WriteDescriptorSetBuilder;
    fn len(&self) -> usize;
}

pub use self::image::*;
pub use acceleration_structure::*;
pub use blas::*;
pub use buffer::*;
pub use buffer_list::*;
pub use texture::*;
pub use tlas::*;
pub use uniforms::*;
