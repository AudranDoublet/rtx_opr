use crate::datatypes::DataType;
use ash::vk;

pub struct BufferVariableList {
    buffers: Vec<vk::Buffer>,
    infos: Vec<vk::DescriptorBufferInfo>,
}

impl BufferVariableList {
    pub fn new(buffers: Vec<vk::Buffer>) -> Self {
        BufferVariableList {
            buffers,
            infos: vec![],
        }
    }

    pub fn empty(size: usize) -> Self {
        let buffers = (0..size).map(|_| vk::Buffer::null()).collect();

        BufferVariableList {
            buffers,
            infos: vec![],
        }
    }
}

impl DataType for BufferVariableList {
    fn write_descriptor_builder(&mut self) -> vk::WriteDescriptorSetBuilder {
        self.infos = self
            .buffers
            .iter()
            .map(|&var| {
                vk::DescriptorBufferInfo::builder()
                    .buffer(var)
                    .range(vk::WHOLE_SIZE)
                    .build()
            })
            .collect();

        vk::WriteDescriptorSet::builder().buffer_info(&self.infos)
    }

    fn len(&self) -> usize {
        self.buffers.len()
    }
}
