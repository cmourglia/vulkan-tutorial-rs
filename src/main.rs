mod util;

use ash::{
    extensions::ext::DebugReport,
    version::{DeviceV1_0, EntryV1_0, InstanceV1_0},
};
use ash::{vk, Device, Entry, Instance};
use std::{
    ffi::{CStr, CString},
    os::raw::{c_char, c_void},
};

const ENABLE_VALIDATION_LAYERS: bool = true;
const REQUIRED_LAYERS: [&'static str; 1] = ["VK_LAYER_LUNARG_standard_validation"];

unsafe extern "system" fn vulkan_debug_callback(
    flag: vk::DebugReportFlagsEXT,
    typ: vk::DebugReportObjectTypeEXT,
    _: u64,
    _: usize,
    _: i32,
    _: *const c_char,
    p_message: *const c_char,
    _: *mut c_void,
) -> u32 {
    if flag == vk::DebugReportFlagsEXT::DEBUG {
        log::debug!("{} - {:?}", typ, CStr::from_ptr(p_message));
    } else if flag == vk::DebugReportFlagsEXT::INFORMATION {
        log::info!("{} - {:?}", typ, CStr::from_ptr(p_message));
    } else if flag == vk::DebugReportFlagsEXT::WARNING {
        log::warn!("{} - {:?}", typ, CStr::from_ptr(p_message));
    } else if flag == vk::DebugReportFlagsEXT::PERFORMANCE_WARNING {
        log::warn!("{} - {:?}", typ, CStr::from_ptr(p_message));
    } else {
        log::error!("{} - {:?}", typ, CStr::from_ptr(p_message));
    }
    vk::FALSE
}

struct VulkanApp {
    _entry: Entry,
    instance: Instance,
    debug_report_callback: Option<(DebugReport, vk::DebugReportCallbackEXT)>,
    _physical_device: vk::PhysicalDevice,
    device: Device,
    _graphics_queue: vk::Queue,
}

impl VulkanApp {
    fn new() -> Self {
        log::debug!("Creating application.");

        let entry = ash::Entry::new().expect("Failed to create entry.");
        let instance = Self::create_instance(&entry);
        let debug_report_callback = Self::setup_debug_messenger(&entry, &instance);
        let physical_device = Self::pick_physical_device(&instance);
        let (device, graphics_queue) =
            Self::create_logical_device_with_graphics_queue(&instance, physical_device);

        Self {
            _entry: entry,
            instance,
            debug_report_callback,
            _physical_device: physical_device,
            device,
            _graphics_queue: graphics_queue,
        }
    }

    fn create_instance(entry: &Entry) -> Instance {
        let app_name = CString::new("Vulkan Application").unwrap();
        let engine_name = CString::new("No Engine").unwrap();
        let app_info = vk::ApplicationInfo::builder()
            .application_name(app_name.as_c_str())
            .application_version(ash::vk_make_version!(0, 1, 0))
            .engine_name(engine_name.as_c_str())
            .engine_version(ash::vk_make_version!(0, 1, 0))
            .api_version(ash::vk_make_version!(1, 0, 0))
            .build();

        let mut extension_names = util::required_extension_names();
        if ENABLE_VALIDATION_LAYERS {
            extension_names.push(DebugReport::name().as_ptr());
        }

        let (_layer_names, layer_names_ptrs) = Self::get_layer_names_and_pointers();

        let mut instance_create_info = vk::InstanceCreateInfo::builder()
            .application_info(&app_info)
            .enabled_extension_names(&extension_names);
        if ENABLE_VALIDATION_LAYERS {
            Self::check_validation_layer_support(&entry);
            instance_create_info = instance_create_info.enabled_layer_names(&layer_names_ptrs);
        }

        unsafe { entry.create_instance(&instance_create_info, None).unwrap() }
    }

    fn get_layer_names_and_pointers() -> (Vec<CString>, Vec<*const i8>) {
        let layer_names = REQUIRED_LAYERS
            .iter()
            .map(|name| CString::new(*name).expect("Failed to build CString"))
            .collect::<Vec<_>>();
        let layer_names_ptrs = layer_names
            .iter()
            .map(|name| name.as_ptr())
            .collect::<Vec<_>>();
        (layer_names, layer_names_ptrs)
    }

    fn check_validation_layer_support(entry: &Entry) {
        for required in REQUIRED_LAYERS.iter() {
            let found = entry
                .enumerate_instance_layer_properties()
                .unwrap()
                .iter()
                .any(|layer| {
                    let name = unsafe { CStr::from_ptr(layer.layer_name.as_ptr()) };
                    let name = name.to_str().expect("Failed to get layer name pointer");
                    required == &name
                });

            if !found {
                panic!("Validation layer not supported: {}", required);
            }
        }
    }

    fn setup_debug_messenger(
        entry: &Entry,
        instance: &Instance,
    ) -> Option<(DebugReport, vk::DebugReportCallbackEXT)> {
        if !ENABLE_VALIDATION_LAYERS {
            return None;
        }
        let create_info = vk::DebugReportCallbackCreateInfoEXT::builder()
            .flags(vk::DebugReportFlagsEXT::all())
            .pfn_callback(Some(vulkan_debug_callback))
            .build();
        let debug_report = DebugReport::new(entry, instance);
        let debug_report_callback = unsafe {
            debug_report
                .create_debug_report_callback(&create_info, None)
                .unwrap()
        };
        Some((debug_report, debug_report_callback))
    }

    fn pick_physical_device(instance: &Instance) -> vk::PhysicalDevice {
        let devices = unsafe { instance.enumerate_physical_devices().unwrap() };
        let device = devices
            .into_iter()
            .find(|device| Self::is_device_suitable(instance, *device))
            .expect("No suitable physical device.");

        let props = unsafe { instance.get_physical_device_properties(device) };
        log::debug!("Selected physical device: {:?}", unsafe {
            CStr::from_ptr(props.device_name.as_ptr())
        });
        device
    }

    fn is_device_suitable(instance: &Instance, device: vk::PhysicalDevice) -> bool {
        Self::find_queue_families(instance, device).is_some()
    }

    fn find_queue_families(instance: &Instance, device: vk::PhysicalDevice) -> Option<u32> {
        let props = unsafe { instance.get_physical_device_queue_family_properties(device) };
        props
            .iter()
            .enumerate()
            .find(|(_, family)| {
                family.queue_count > 0 && family.queue_flags.contains(vk::QueueFlags::GRAPHICS)
            })
            .map(|(index, _)| index as _)
    }

    fn create_logical_device_with_graphics_queue(
        instance: &Instance,
        device: vk::PhysicalDevice,
    ) -> (Device, vk::Queue) {
        let queue_family_index = Self::find_queue_families(instance, device).unwrap();
        let queue_priorities = [1.0f32];
        let queue_create_infos = [vk::DeviceQueueCreateInfo::builder()
            .queue_family_index(queue_family_index)
            .queue_priorities(&queue_priorities)
            .build()];

        let device_features = vk::PhysicalDeviceFeatures::builder().build();

        let (_layer_names, layer_names_ptrs) = Self::get_layer_names_and_pointers();

        let mut device_create_info_builder = vk::DeviceCreateInfo::builder()
            .queue_create_infos(&queue_create_infos)
            .enabled_features(&device_features);
        if ENABLE_VALIDATION_LAYERS {
            device_create_info_builder =
                device_create_info_builder.enabled_layer_names(&layer_names_ptrs)
        }
        let device_create_info = device_create_info_builder.build();

        let device = unsafe {
            instance
                .create_device(device, &device_create_info, None)
                .expect("Failed to create logical device.")
        };
        let graphics_queue = unsafe { device.get_device_queue(queue_family_index, 0) };

        (device, graphics_queue)
    }

    fn run(&mut self) {
        log::debug!("Running application.");
    }
}

impl Drop for VulkanApp {
    fn drop(&mut self) {
        log::debug!("Dropping application.");
        unsafe {
            self.device.destroy_device(None);
            if let Some((report, callback)) = self.debug_report_callback.take() {
                report.destroy_debug_report_callback(callback, None);
            }
            self.instance.destroy_instance(None);
        }
    }
}

fn main() {
    env_logger::init();
    VulkanApp::new().run()
}
