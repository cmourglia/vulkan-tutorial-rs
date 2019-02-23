use ash::{
    extensions::{ext::DebugReport, khr::Surface},
    version::{DeviceV1_0, InstanceV1_0},
    vk, Device, Entry, Instance,
};

pub struct VkContext {
    _entry: Entry,
    instance: Instance,
    debug_report_callback: Option<(DebugReport, vk::DebugReportCallbackEXT)>,
    surface: Surface,
    surface_khr: vk::SurfaceKHR,
    physical_device: vk::PhysicalDevice,
    device: Device,
    queue_families_indices: QueueFamiliesIndices,
    graphics_queue: vk::Queue,
    present_queue: vk::Queue,
    general_command_pool: vk::CommandPool,
    transient_command_pool: vk::CommandPool,
}

impl VkContext {
    pub fn new(
        entry: Entry,
        instance: Instance,
        debug_report_callback: Option<(DebugReport, vk::DebugReportCallbackEXT)>,
        surface: Surface,
        surface_khr: vk::SurfaceKHR,
        physical_device: vk::PhysicalDevice,
        device: Device,
        queue_families_indices: QueueFamiliesIndices,
        graphics_queue: vk::Queue,
        present_queue: vk::Queue,
        general_command_pool: vk::CommandPool,
        transient_command_pool: vk::CommandPool,
    ) -> Self {
        VkContext {
            _entry: entry,
            instance,
            debug_report_callback,
            surface,
            surface_khr,
            physical_device,
            device,
            queue_families_indices,
            graphics_queue,
            present_queue,
            general_command_pool,
            transient_command_pool,
        }
    }
}

impl VkContext {
    pub fn instance(&self) -> &Instance {
        &self.instance
    }

    pub fn surface(&self) -> &Surface {
        &self.surface
    }

    pub fn surface_khr(&self) -> vk::SurfaceKHR {
        self.surface_khr
    }

    pub fn physical_device(&self) -> vk::PhysicalDevice {
        self.physical_device
    }

    pub fn device(&self) -> &Device {
        &self.device
    }

    pub fn queue_families_indices(&self) -> QueueFamiliesIndices {
        self.queue_families_indices
    }

    pub fn graphics_queue(&self) -> vk::Queue {
        self.graphics_queue
    }

    pub fn present_queue(&self) -> vk::Queue {
        self.present_queue
    }

    pub fn general_command_pool(&self) -> vk::CommandPool {
        self.general_command_pool
    }
}

impl VkContext {
    pub fn get_mem_properties(&self) -> vk::PhysicalDeviceMemoryProperties {
        unsafe {
            self.instance
                .get_physical_device_memory_properties(self.physical_device)
        }
    }

    /// Find the first compatible format from `candidates`.
    pub fn find_supported_format(
        &self,
        candidates: &[vk::Format],
        tiling: vk::ImageTiling,
        features: vk::FormatFeatureFlags,
    ) -> Option<vk::Format> {
        candidates.iter().cloned().find(|candidate| {
            let props = unsafe {
                self.instance
                    .get_physical_device_format_properties(self.physical_device, *candidate)
            };
            (tiling == vk::ImageTiling::LINEAR && props.linear_tiling_features.contains(features))
                || (tiling == vk::ImageTiling::OPTIMAL
                    && props.optimal_tiling_features.contains(features))
        })
    }

    /// Return the maximim sample count supported.
    pub fn get_max_usable_sample_count(&self) -> vk::SampleCountFlags {
        let props = unsafe {
            self.instance
                .get_physical_device_properties(self.physical_device)
        };
        let color_sample_counts = props.limits.framebuffer_color_sample_counts;
        let depth_sample_counts = props.limits.framebuffer_depth_sample_counts;
        let sample_counts = color_sample_counts.min(depth_sample_counts);

        if sample_counts.contains(vk::SampleCountFlags::TYPE_64) {
            vk::SampleCountFlags::TYPE_64
        } else if sample_counts.contains(vk::SampleCountFlags::TYPE_32) {
            vk::SampleCountFlags::TYPE_32
        } else if sample_counts.contains(vk::SampleCountFlags::TYPE_16) {
            vk::SampleCountFlags::TYPE_16
        } else if sample_counts.contains(vk::SampleCountFlags::TYPE_8) {
            vk::SampleCountFlags::TYPE_8
        } else if sample_counts.contains(vk::SampleCountFlags::TYPE_4) {
            vk::SampleCountFlags::TYPE_4
        } else if sample_counts.contains(vk::SampleCountFlags::TYPE_2) {
            vk::SampleCountFlags::TYPE_2
        } else {
            vk::SampleCountFlags::TYPE_1
        }
    }

    /// Create a one time use command buffer and pass it to `executor`.
    pub fn execute_one_time_commands<F: FnOnce(vk::CommandBuffer)>(&self, executor: F) {
        let command_buffer = {
            let alloc_info = vk::CommandBufferAllocateInfo::builder()
                .level(vk::CommandBufferLevel::PRIMARY)
                .command_pool(self.transient_command_pool)
                .command_buffer_count(1)
                .build();

            unsafe { self.device.allocate_command_buffers(&alloc_info).unwrap()[0] }
        };
        let command_buffers = [command_buffer];

        // Begin recording
        {
            let begin_info = vk::CommandBufferBeginInfo::builder()
                .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT)
                .build();
            unsafe {
                self.device
                    .begin_command_buffer(command_buffer, &begin_info)
                    .unwrap()
            };
        }

        // Execute user function
        executor(command_buffer);

        // End recording
        unsafe { self.device.end_command_buffer(command_buffer).unwrap() };

        // Submit and wait
        {
            let submit_info = vk::SubmitInfo::builder()
                .command_buffers(&command_buffers)
                .build();
            let submit_infos = [submit_info];
            unsafe {
                self.device
                    .queue_submit(self.graphics_queue, &submit_infos, vk::Fence::null())
                    .unwrap();
                self.device.queue_wait_idle(self.graphics_queue).unwrap();
            };
        }

        // Free
        unsafe {
            self.device
                .free_command_buffers(self.transient_command_pool, &command_buffers)
        };
    }
}

impl Drop for VkContext {
    fn drop(&mut self) {
        unsafe {
            self.device
                .destroy_command_pool(self.transient_command_pool, None);
            self.device
                .destroy_command_pool(self.general_command_pool, None);
            self.device.destroy_device(None);
            self.surface.destroy_surface(self.surface_khr, None);
            if let Some((report, callback)) = self.debug_report_callback.take() {
                report.destroy_debug_report_callback(callback, None);
            }
            self.instance.destroy_instance(None);
        }
    }
}

/// Find a memory type in `mem_properties` that is suitable
/// for `requirements` and supports `required_properties`.
///
/// # Returns
///
/// The index of the memory type from `mem_properties`.
pub fn find_memory_type(
    requirements: vk::MemoryRequirements,
    mem_properties: vk::PhysicalDeviceMemoryProperties,
    required_properties: vk::MemoryPropertyFlags,
) -> u32 {
    for i in 0..mem_properties.memory_type_count {
        if requirements.memory_type_bits & (1 << i) != 0
            && mem_properties.memory_types[i as usize]
                .property_flags
                .contains(required_properties)
        {
            return i;
        }
    }
    panic!("Failed to find suitable memory type.")
}

#[derive(Clone, Copy)]
pub struct QueueFamiliesIndices {
    pub graphics_index: u32,
    pub present_index: u32,
}
