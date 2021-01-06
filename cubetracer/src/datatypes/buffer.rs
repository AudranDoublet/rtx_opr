use std::ffi::c_void;
use std::mem::size_of;
use std::sync::Arc;

use ash::{util::Align, vk::DeviceSize};
use ash::{version::DeviceV1_0, vk};

use crate::context::{find_memory_type, Context};

use crate::datatypes::DataType;

/// Wrapper over a raw pointer to make it moveable and accessible from other threads
struct MemoryMapPointer(*mut c_void);
unsafe impl Send for MemoryMapPointer {}
unsafe impl Sync for MemoryMapPointer {}

pub struct BufferVariable {
    name: String,
    context: Arc<Context>,
    buffer: vk::Buffer,
    memory: vk::DeviceMemory,
    size: vk::DeviceSize,
    element_count: usize,

    mapped_pointer: Option<MemoryMapPointer>,
    info: Vec<vk::DescriptorBufferInfo>,
}

impl BufferVariable {
    pub fn create(
        name: String,
        context: &Arc<Context>,
        size: u64,
        element_count: usize,
        usage: vk::BufferUsageFlags,
        mem_properties: vk::MemoryPropertyFlags,
    ) -> BufferVariable {
        let device = context.device();
        let buffer = {
            let buffer_info = vk::BufferCreateInfo::builder()
                .size(size)
                .usage(usage)
                .sharing_mode(vk::SharingMode::EXCLUSIVE);
            unsafe {
                device
                    .create_buffer(&buffer_info, None)
                    .expect("Failed to create buffer")
            }
        };

        let mem_requirements = unsafe { device.get_buffer_memory_requirements(buffer) };
        let memory = {
            let mem_type = find_memory_type(
                mem_requirements,
                context.get_mem_properties(),
                mem_properties,
            );

            let alloc_info = vk::MemoryAllocateInfo::builder()
                .allocation_size(mem_requirements.size)
                .memory_type_index(mem_type);
            unsafe {
                device
                    .allocate_memory(&alloc_info, None)
                    .expect("Failed to allocate memory")
            }
        };

        unsafe {
            device
                .bind_buffer_memory(buffer, memory, 0)
                .expect("Failed to bind buffer memory")
        };

        BufferVariable {
            name,
            context: Arc::clone(context),
            buffer,
            memory,
            size,
            element_count,
            mapped_pointer: None,

            info: Vec::new(),
        }
    }

    pub fn null(name: String, context: &Arc<Context>) -> BufferVariable {
        BufferVariable {
            name,
            context: Arc::clone(context),
            buffer: vk::Buffer::null(),
            memory: vk::DeviceMemory::null(),
            size: 0,
            element_count: 0,
            mapped_pointer: None,
            info: Vec::new(),
        }
    }

    pub fn empty_device_buffer(
        name: String,
        context: &Arc<Context>,
        usage: vk::BufferUsageFlags,
        size: usize,
    ) -> BufferVariable {
        Self::create(
            name,
            context,
            size as u64,
            size,
            vk::BufferUsageFlags::TRANSFER_DST | usage,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
        )
    }

    pub fn device_buffer<T: Copy>(
        name: String,
        context: &Arc<Context>,
        usage: vk::BufferUsageFlags,
        data: &[T],
    ) -> (BufferVariable, BufferVariable) {
        context.execute_one_time_commands(|command_buffer| {
            let size = (data.len() * size_of::<T>()) as vk::DeviceSize;
            let staging_buffer = Self::host_buffer(
                format!("{} host-copy", name),
                context,
                vk::BufferUsageFlags::TRANSFER_SRC,
                data,
            );

            let buffer = Self::create(
                name,
                context,
                size,
                data.len(),
                vk::BufferUsageFlags::TRANSFER_DST | usage,
                vk::MemoryPropertyFlags::DEVICE_LOCAL,
            );

            buffer.cmd_copy(command_buffer, &staging_buffer, staging_buffer.size);
            (buffer, staging_buffer)
        })
    }

    pub fn empty_host_buffer(
        name: String,
        context: &Arc<Context>,
        usage: vk::BufferUsageFlags,
        size: usize,
    ) -> BufferVariable {
        Self::create(
            name,
            context,
            size as u64,
            size,
            vk::BufferUsageFlags::TRANSFER_SRC | usage,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        )
    }

    pub fn host_buffer<T: Copy>(
        name: String,
        context: &Arc<Context>,
        usage: vk::BufferUsageFlags,
        data: &[T],
    ) -> BufferVariable {
        let size = (data.len() * size_of::<T>()) as vk::DeviceSize;

        let mut buffer = Self::create(
            name,
            context,
            size,
            data.len(),
            usage,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        );

        buffer.set_host(data);
        buffer
    }

    pub fn set_host<T: Copy>(&mut self, data: &[T]) {
        unsafe {
            let data_ptr = self.map_memory();
            mem_copy(data_ptr, data);
        };
    }

    pub fn set_device<T: Copy>(&mut self, context: &Arc<Context>, data: &[T]) {
        context.execute_one_time_commands(|command_buffer| {
            let staging_buffer = Self::host_buffer(
                format!("{} host-copy set", self.name),
                context,
                vk::BufferUsageFlags::TRANSFER_SRC,
                data,
            );
            self.cmd_copy(command_buffer, &staging_buffer, staging_buffer.size);
        });
    }

    /// Returns the raw vulkan buffer
    pub fn buffer(&self) -> &vk::Buffer {
        &self.buffer
    }

    /// Returns the size in bytes of the buffer
    pub fn size(&self) -> u64 {
        self.size
    }

    /// Returns the element count of the buffer
    pub fn element_count(&self) -> usize {
        self.element_count
    }

    /// Register the commands to copy the `size` first bytes of `src` this buffer.
    pub fn cmd_copy(&self, command_buffer: vk::CommandBuffer, src: &Self, size: vk::DeviceSize) {
        let region = vk::BufferCopy {
            src_offset: 0,
            dst_offset: 0,
            size,
        };
        let regions = [region];

        unsafe {
            self.context
                .device()
                .cmd_copy_buffer(command_buffer, src.buffer, self.buffer, &regions)
        };
    }

    /// Map the buffer memory and return the mapped pointer.
    ///
    /// If the memory is already mapped it just returns the pointer.
    pub fn map_memory(&mut self) -> *mut c_void {
        if let Some(ptr) = &self.mapped_pointer {
            ptr.0
        } else {
            unsafe {
                let ptr = self
                    .context
                    .device()
                    .map_memory(self.memory, 0, self.size, vk::MemoryMapFlags::empty())
                    .expect("Failed to map memory");
                self.mapped_pointer = Some(MemoryMapPointer(ptr));
                ptr
            }
        }
    }

    /// Unmap the buffer memory.
    ///
    /// Does nothing if memory is not mapped.
    pub fn unmap_memory(&mut self) {
        if self.mapped_pointer.take().is_some() {
            unsafe {
                self.context.device().unmap_memory(self.memory);
            }
        }
    }
}

impl DataType for BufferVariable {
    fn write_descriptor_builder(&mut self) -> vk::WriteDescriptorSetBuilder {
        self.info = vec![vk::DescriptorBufferInfo::builder()
            .buffer(self.buffer)
            .range(vk::WHOLE_SIZE)
            .build()];

        vk::WriteDescriptorSet::builder().buffer_info(&self.info)
    }

    fn len(&self) -> usize {
        1
    }
}

impl Drop for BufferVariable {
    fn drop(&mut self) {
        unsafe {
            self.unmap_memory();
            self.context.device().destroy_buffer(self.buffer, None);
            self.context.device().free_memory(self.memory, None);
        }
    }
}

/// Utility function that copy the content of a slice at the position of a given pointer.
pub unsafe fn mem_copy<T: Copy>(ptr: *mut c_void, data: &[T]) {
    let elem_size = size_of::<T>() as DeviceSize;
    let size = data.len() as DeviceSize * elem_size;
    let mut align = Align::new(ptr, elem_size, size);
    align.copy_from_slice(data);
}
