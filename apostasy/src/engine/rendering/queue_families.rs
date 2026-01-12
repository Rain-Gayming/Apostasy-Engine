use anyhow::Result;
use ash::vk;

use crate::engine::rendering::physical_device::PhysicalDevice;

pub struct QueueFamily {
    pub index: u32,
    pub properties: Vec<vk::QueueFamilyProperties>,
}

pub type QueueFamilyPicker = fn(Vec<PhysicalDevice>) -> Result<(PhysicalDevice, QueueFamilies)>;
pub struct QueueFamilies {
    pub graphics: QueueFamily,
    pub present: QueueFamily,
    pub transfer: QueueFamily,
    pub compute: QueueFamily,
}
