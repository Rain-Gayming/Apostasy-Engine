use ash::vk;

pub struct Surface {
    pub handle: vk::SurfaceKHR,
    pub capabilities: vk::SurfaceCapabilitiesKHR,
}
