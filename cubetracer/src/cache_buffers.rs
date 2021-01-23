use ash::vk;

use std::sync::Arc;
use std::collections::HashMap;

use crate::context::Context;
use crate::datatypes::*;
use crate::descriptors::*;
use crate::pipeline::*;
use crate::window::*;

pub enum BufferFormat {
    RGBA,
    VALUE,
}

impl BufferFormat {
    pub fn vulkan(&self) -> vk::Format {
        match self {
            BufferFormat::RGBA => vk::Format::R32G32B32A32_SFLOAT,
            BufferFormat::VALUE => vk::Format::R32_SFLOAT,
        }
    }
}

trait CacheBuffer {
    fn swap(&mut self) -> bool;

    fn bindings<'a>(&'a mut self) -> Vec<&'a mut dyn DataType>;

    fn texture<'a>(&'a self) -> &'a TextureVariable;

    fn buffers(&self) -> Vec<vk::Image>;
}

/////// SIMPLE BUFFERING
struct SimpleCacheBuffer {
    buffer: TextureVariable,
}

impl CacheBuffer for SimpleCacheBuffer {
    fn swap(&mut self) -> bool {
        false
    }

    fn bindings<'a>(&'a mut self) -> Vec<&'a mut dyn DataType> {
        vec![&mut self.buffer]
    }

    fn buffers(&self) -> Vec<vk::Image> {
        vec![self.buffer.image.image]
    }

    fn texture<'a>(&'a self) -> &'a TextureVariable {
        &self.buffer
    }
}

/////// DOUBLE BUFFERING
struct DoubleBufferingCache {
    a: TextureVariable,
    b: TextureVariable,
}

impl CacheBuffer for DoubleBufferingCache {
    fn swap(&mut self) -> bool {
        std::mem::swap(&mut self.a, &mut self.b);
        true
    }

    fn bindings<'a>(&'a mut self) -> Vec<&'a mut dyn DataType> {
        vec![&mut self.a, &mut self.b]
    }

    fn buffers(&self) -> Vec<vk::Image> {
        vec![self.a.image.image, self.b.image.image]
    }

    fn texture<'a>(&'a self) -> &'a TextureVariable {
        &self.a
    }
}

/////// BUFFER LIST
pub struct BufferList {
    context: Arc<Context>,
    buffers: Vec<Box<dyn CacheBuffer>>,
    names: HashMap<String, usize>,
}

impl BufferList {
    pub fn new(context: &Arc<Context>) -> Self {
        BufferList {
            context: Arc::clone(context),
            buffers: vec![],
            names: HashMap::new(),
        }
    }

    pub fn descriptor_set(&mut self, shader_types: &[ShaderType]) -> DescriptorSet {
        let mut builder = DescriptorSetBuilder::new(&self.context);

        for buffer in &mut self.buffers {
            builder.bindings(
                vk::DescriptorType::STORAGE_IMAGE,
                buffer.bindings(),
                shader_types
            );
        }

        builder.build()
    }

    pub fn update(&mut self, descriptor_set: &DescriptorSet) {
        let mut builder = descriptor_set.update(&self.context);
        let mut i = 0;

        for buffer in &mut self.buffers {
            if buffer.swap() {
                for val in buffer.bindings() {
                    builder.register(i, vk::DescriptorType::STORAGE_IMAGE, val);
                    i += 1;
                }
            } else {
                i += 1;
            }
        }

        builder.update();
    }

    pub fn images(&self, caches: &[&str]) -> Vec<vk::Image> {
        let mut buffers = vec![];

        for name in caches {
            let name = name.to_string();

            for buffer in self.buffers[self.names[&name]].buffers() {
                buffers.push(buffer);
            }
        }

        buffers
    }

    pub fn texture(&self, name: &str) -> &TextureVariable {
        self.buffers[self.names[name]].texture()
    }

    pub fn simple_same(&mut self, name: &str, swapchain: &Swapchain) -> &mut Self {
        let buffer = TextureVariable::from_swapchain(
            &self.context,
            swapchain,
        );

        self.names.insert(name.to_string(), self.buffers.len());
        self.buffers.push(Box::new(
            SimpleCacheBuffer {
                buffer,
            }
        ));

        self
    }

    pub fn simple(&mut self, name: &str, swapchain: &Swapchain, format: BufferFormat) -> &mut Self {
        let buffer = TextureVariable::from_swapchain_format(
            &self.context,
            swapchain,
            format.vulkan(),
        );

        self.names.insert(name.to_string(), self.buffers.len());
        self.buffers.push(Box::new(
            SimpleCacheBuffer {
                buffer,
            }
        ));

        self
    }

    pub fn double(&mut self, name: &str, swapchain: &Swapchain, format: BufferFormat) -> &mut Self {
        let a = TextureVariable::from_swapchain_format(
            &self.context,
            swapchain,
            format.vulkan(),
        );

        let b = TextureVariable::from_swapchain_format(
            &self.context,
            swapchain,
            format.vulkan(),
        );

        self.names.insert(name.to_string(), self.buffers.len());
        self.buffers.push(Box::new(
            DoubleBufferingCache {
                a,
                b,
            }
        ));

        self
    }
}
