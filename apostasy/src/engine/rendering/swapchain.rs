use std::sync::Arc;

use anyhow::Result;
use ash::vk::{self, Handle};
use winit::window::Window;

use crate::engine::rendering::{rendering_context::RenderingContext, surface::Surface};

pub struct Swapchain {
    pub desired_image_count: u32,
    pub format: vk::Format,
    pub extent: vk::Extent2D,
    pub image_views: Vec<vk::ImageView>,
    pub images: Vec<vk::Image>,
    pub handle: vk::SwapchainKHR,
    pub surface: Surface,
    pub window: Arc<Window>,
    pub context: Arc<RenderingContext>,
    pub is_dirty: bool,
    pub depth_format: vk::Format,
    pub depth_image: vk::Image,
    pub depth_image_view: vk::ImageView,
    pub depth_memory: vk::DeviceMemory,
}

impl Swapchain {
    /// Creates a new Swapchain
    pub fn new(context: Arc<RenderingContext>, window: Arc<Window>) -> Result<Self> {
        unsafe {
            let surface = context.create_surface(&window)?;
            let format = vk::Format::B8G8R8A8_SRGB;
            let depth_format = vk::Format::D32_SFLOAT;
            let extent = if surface.capabilities.current_extent.width != u32::MAX {
                surface.capabilities.current_extent
            } else {
                vk::Extent2D {
                    width: window.inner_size().width,
                    height: window.inner_size().height,
                }
            };
            let image_count = (surface.capabilities.min_image_count + 1).clamp(
                surface.capabilities.min_image_count,
                if surface.capabilities.max_image_count != 0 {
                    surface.capabilities.max_image_count
                } else {
                    u32::MAX
                },
            );

            Ok(Self {
                desired_image_count: image_count,
                format,
                extent,
                image_views: Vec::new(),
                images: Vec::new(),
                handle: Default::default(),
                surface,
                window,
                context,
                is_dirty: true,
                depth_format,
                depth_image: vk::Image::null(),
                depth_image_view: vk::ImageView::null(),
                depth_memory: vk::DeviceMemory::null(),
            })
        }
    }

    /// Resizes the swapchain based on the window size
    pub fn resize(&mut self) -> Result<()> {
        let size = self.window.inner_size();
        self.extent = vk::Extent2D {
            width: size.width,
            height: size.height,
        };

        if self.extent.width == 0 || self.extent.height == 0 {
            return Ok(());
        }

        unsafe {
            self.context.device.device_wait_idle()?;
            let new_swapchain = self.context.swapchain_extensions.create_swapchain(
                &vk::SwapchainCreateInfoKHR::default()
                    .surface(self.surface.handle)
                    .min_image_count(self.desired_image_count)
                    .image_format(self.format)
                    .image_color_space(vk::ColorSpaceKHR::SRGB_NONLINEAR)
                    .image_extent(self.extent)
                    .image_array_layers(1)
                    .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
                    .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
                    .pre_transform(vk::SurfaceTransformFlagsKHR::IDENTITY)
                    .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
                    .present_mode(vk::PresentModeKHR::FIFO)
                    .clipped(true)
                    .old_swapchain(self.handle),
                None,
            )?;

            for image_view in self.image_views.drain(..) {
                self.context.device.destroy_image_view(image_view, None);
            }

            if !self.depth_image_view.is_null() {
                self.context
                    .device
                    .destroy_image_view(self.depth_image_view, None);
                self.depth_image_view = vk::ImageView::null();
            }
            if !self.depth_image.is_null() {
                self.context.device.destroy_image(self.depth_image, None);
                self.depth_image = vk::Image::null();
            }
            if !self.depth_memory.is_null() {
                self.context.device.free_memory(self.depth_memory, None);
                self.depth_memory = vk::DeviceMemory::null();
            }

            self.context
                .swapchain_extensions
                .destroy_swapchain(self.handle, None);

            self.images.clear();
            self.handle = new_swapchain;
            self.images = self
                .context
                .swapchain_extensions
                .get_swapchain_images(new_swapchain)?;

            for image in &self.images {
                self.image_views.push(self.context.create_image_view(
                    *image,
                    self.format,
                    vk::ImageAspectFlags::COLOR,
                )?);
            }

            // Create depth buffer
            self.depth_image = self.context.create_image(
                self.extent,
                self.depth_format,
                vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT,
            )?;
            self.depth_memory = self
                .context
                .allocate_image_memory(self.depth_image, vk::MemoryPropertyFlags::DEVICE_LOCAL)?;
            self.depth_image_view = self.context.create_image_view(
                self.depth_image,
                self.depth_format,
                vk::ImageAspectFlags::DEPTH,
            )?;
        }

        self.is_dirty = false;
        Ok(())
    }

    /// Acquires the next image in the swapchain
    pub fn acquire_next_image(&mut self, image_available_semaphore: vk::Semaphore) -> Result<u32> {
        let (image_index, is_suboptimal) = unsafe {
            self.context.swapchain_extensions.acquire_next_image(
                self.handle,
                u64::MAX,
                image_available_semaphore,
                vk::Fence::null(),
            )?
        };

        if is_suboptimal {
            self.is_dirty = true;
        }

        Ok(image_index)
    }

    /// Presents an image to the renderer
    pub fn present_image(
        &mut self,
        image_index: u32,
        render_finished_semaphore: vk::Semaphore,
    ) -> Result<()> {
        let is_suboptimal = unsafe {
            self.context.swapchain_extensions.queue_present(
                self.context.queues[self.context.queue_families.present as usize],
                &vk::PresentInfoKHR::default()
                    .wait_semaphores(&[render_finished_semaphore])
                    .swapchains(&[self.handle])
                    .image_indices(&[image_index]),
            )
        }?;

        if is_suboptimal {
            self.is_dirty = true;
        }

        Ok(())
    }
}
impl Drop for Swapchain {
    fn drop(&mut self) {
        unsafe {
            if !self.depth_image_view.is_null() {
                self.context
                    .device
                    .destroy_image_view(self.depth_image_view, None);
            }
            if !self.depth_image.is_null() {
                self.context.device.destroy_image(self.depth_image, None);
            }
            if !self.depth_memory.is_null() {
                self.context.device.free_memory(self.depth_memory, None);
            }
            for image_view in &self.image_views {
                self.context.device.destroy_image_view(*image_view, None);
            }
            self.context
                .swapchain_extensions
                .destroy_swapchain(self.handle, None);
        }
    }
}
