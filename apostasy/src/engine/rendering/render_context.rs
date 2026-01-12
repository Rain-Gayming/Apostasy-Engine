use anyhow::Result;
use ash::{
    khr::{surface, swapchain},
    vk::{
        self, ApplicationInfo, DeviceQueueCreateInfo, InstanceCreateInfo,
        PhysicalDeviceBufferDeviceAddressFeatures, PhysicalDeviceDynamicRenderingFeatures, Queue,
    },
};
use std::{collections::HashSet, sync::Arc};
use winit::{
    raw_window_handle::{HasDisplayHandle, HasWindowHandle},
    window::Window,
};

use crate::engine::rendering::{
    physical_device::PhysicalDevice,
    queue_families::{QueueFamilies, QueueFamily, QueueFamilyPicker},
};

pub struct RenderContext {
    pub queues: Vec<vk::Queue>,
    pub device: vk::Device,
    pub queue_family_indices: HashSet<u32>,
    pub queue_families: QueueFamilies,
    pub physical_device: PhysicalDevice,
    pub surface: vk::SurfaceKHR,
    pub surface_extensions: ash::khr::surface::Instance,
    pub instance: ash::Instance,
    pub entry: ash::Entry,
    pub context_attributes: RenderContextAttributes,
}
pub struct RenderContextAttributes {
    pub window: Arc<Window>,
    pub queue_family_picker: QueueFamilyPicker,
}

impl RenderContext {
    pub fn new(context_attributes: RenderContextAttributes) -> Result<Self> {
        unsafe {
            let entry = ash::Entry::load()?;

            let window = context_attributes.window.clone();

            let raw_display_handle = window.display_handle()?.as_raw();
            let raw_window_handle = window.window_handle()?.as_raw();

            let instance = entry.create_instance(
                &InstanceCreateInfo::default()
                    .application_info(&ApplicationInfo::default().api_version(vk::API_VERSION_1_3))
                    .enabled_extension_names(ash_window::enumerate_required_extensions(
                        raw_display_handle,
                    )?),
                None,
            )?;

            let surface_extensions = surface::Instance::new(&entry, &instance);
            let surface = ash_window::create_surface(
                &entry,
                &instance,
                raw_display_handle,
                raw_window_handle,
                None,
            )?;

            let mut physical_devices = instance
                .enumerate_physical_devices()?
                .into_iter()
                .map(|handle| {
                    let properties = instance.get_physical_device_properties(handle);
                    let features = instance.get_physical_device_features(handle);
                    let memory_properties = instance.get_physical_device_memory_properties(handle);
                    let queue_families =
                        instance.get_physical_device_queue_family_properties(handle);

                    let queue_families = queue_families
                        .iter()
                        .cloned()
                        .enumerate()
                        .map(|(index, properties)| QueueFamily {
                            index: index as u32,
                            properties: vec![properties],
                        })
                        .collect::<Vec<QueueFamily>>();

                    PhysicalDevice {
                        handle,
                        properties,
                        features,
                        memory_properties,
                        queue_families,
                    }
                })
                .collect::<Vec<PhysicalDevice>>();

            physical_devices.retain(|device| {
                surface_extensions
                    .get_physical_device_surface_support(device.handle, 0, surface)
                    .unwrap_or(false)
            });

            let (physical_device, queue_families) =
                (context_attributes.queue_family_picker)(physical_devices)?;

            let queue_family_indices: HashSet<u32> = HashSet::from_iter([
                queue_families.graphics.index,
                queue_families.compute.index,
                queue_families.transfer.index,
                queue_families.present.index,
            ]);

            let queue_create_infos = queue_family_indices
                .iter()
                .copied()
                .map(|index| {
                    DeviceQueueCreateInfo::default()
                        .queue_family_index(index)
                        .queue_priorities(&[1.0])
                })
                .collect::<Vec<_>>();

            let device = instance.create_device(
                physical_device.handle,
                &vk::DeviceCreateInfo::default()
                    .queue_create_infos(&queue_create_infos)
                    .enabled_extension_names(&[swapchain::NAME.as_ptr()])
                    .push_next(&mut PhysicalDeviceDynamicRenderingFeatures::default())
                    .push_next(
                        &mut PhysicalDeviceBufferDeviceAddressFeatures::default()
                            .buffer_device_address(true),
                    ),
                None,
            )?;

            let queues = queue_family_indices
                .iter()
                .map(|index| device.get_device_queue(*index, 0))
                .collect::<Vec<Queue>>();

            Ok(Self {
                queues,
                device: Default::default(),
                queue_family_indices,
                queue_families,
                physical_device,
                surface,
                surface_extensions,
                instance,
                entry,
                context_attributes,
            })
        }
    }
}

impl Drop for RenderContext {
    fn drop(&mut self) {
        unsafe {
            self.surface_extensions.destroy_surface(self.surface, None);
        }
    }
}
