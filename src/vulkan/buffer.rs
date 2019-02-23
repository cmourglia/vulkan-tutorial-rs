use super::context::*;
use ash::{version::DeviceV1_0, vk};
use std::rc::Rc;

pub struct Buffer {
    context: Rc<VkContext>,
    pub buffer: vk::Buffer,
    pub memory: vk::DeviceMemory,
    pub size: vk::DeviceSize,
}

impl Buffer {
    fn new(
        context: Rc<VkContext>,
        buffer: vk::Buffer,
        memory: vk::DeviceMemory,
        size: vk::DeviceSize,
    ) -> Self {
        Self {
            context,
            buffer,
            memory,
            size,
        }
    }

    /// Create a buffer and allocate its memory.
    ///
    /// # Returns
    ///
    /// The buffer, its memory and the actual size in bytes of the
    /// allocated memory since in may differ from the requested size.
    pub fn create(
        context: Rc<VkContext>,
        size: vk::DeviceSize,
        usage: vk::BufferUsageFlags,
        mem_properties: vk::MemoryPropertyFlags,
    ) -> Self {
        let device = context.device();
        let buffer = {
            let buffer_info = vk::BufferCreateInfo::builder()
                .size(size)
                .usage(usage)
                .sharing_mode(vk::SharingMode::EXCLUSIVE)
                .build();
            unsafe { device.create_buffer(&buffer_info, None).unwrap() }
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
                .memory_type_index(mem_type)
                .build();
            unsafe { device.allocate_memory(&alloc_info, None).unwrap() }
        };

        unsafe { device.bind_buffer_memory(buffer, memory, 0).unwrap() };

        Buffer::new(context, buffer, memory, mem_requirements.size)
    }
}

impl Buffer {
    /// Copy the `size` first bytes of `src` this buffer.
    ///
    /// It's done using a command buffer allocated from
    /// `command_pool`. The command buffer is cubmitted tp
    /// `transfer_queue`.
    pub fn copy(&self, src: &Buffer, size: vk::DeviceSize) {
        self.context.execute_one_time_commands(|buffer| {
            let region = vk::BufferCopy {
                src_offset: 0,
                dst_offset: 0,
                size,
            };
            let regions = [region];

            unsafe {
                self.context
                    .device()
                    .cmd_copy_buffer(buffer, src.buffer, self.buffer, &regions)
            };
        });
    }
}

impl Drop for Buffer {
    fn drop(&mut self) {
        unsafe {
            self.context.device().destroy_buffer(self.buffer, None);
            self.context.device().free_memory(self.memory, None);
        }
    }
}
