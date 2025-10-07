use std::sync::Arc;

use anyhow::Result;
use ash::vk;
use winit::window::Window;

use crate::app::engine::rendering_context::{RenderingContext, Surface};

pub struct Swapchain {
    pub desired_image_count: u32,
    pub format: vk::Format,
    pub extent: vk::Extent2D,
    pub views: Vec<vk::ImageView>,
    pub images: Vec<vk::Image>,
    handle: vk::SwapchainKHR,
    surface: Surface,
    pub window: Arc<Window>,
    context: Arc<RenderingContext>,
    pub is_dirty: bool,
}

impl Swapchain {
    pub fn new(context: Arc<RenderingContext>, window: Arc<Window>) -> Result<Self> {
        let surface = unsafe { context.create_surface(&window)? };
        let format = vk::Format::B8G8R8A8_SRGB;

        // this fixes it for wayland lol
        let extent = if surface.capabilities.current_extent.width == u32::MAX {
            let size = window.inner_size();
            vk::Extent2D {
                width: size.width,
                height: size.height,
            }
        } else {
            surface.capabilities.current_extent
        };

        let desired_image_count = (surface.capabilities.min_image_count + 1).clamp(
            surface.capabilities.min_image_count,
            if surface.capabilities.max_image_count == 0 {
                u32::MAX
            } else {
                surface.capabilities.max_image_count
            },
        );

        Ok(Self {
            desired_image_count,
            format,
            extent,
            views: Vec::new(),
            images: Vec::new(),
            handle: Default::default(),
            surface,
            window,
            context,
            is_dirty: true,
        })
    }

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
            let new_swapchain = self.context.swapchain_extension.create_swapchain(
                &vk::SwapchainCreateInfoKHR::default()
                    .surface(self.surface.handle)
                    .min_image_count(self.desired_image_count)
                    .image_format(self.format)
                    .image_color_space(vk::ColorSpaceKHR::SRGB_NONLINEAR)
                    .image_extent(self.extent)
                    .image_array_layers(1)
                    .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
                    .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
                    .pre_transform(self.surface.capabilities.current_transform)
                    .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
                    .present_mode(vk::PresentModeKHR::FIFO)
                    .clipped(true)
                    .old_swapchain(self.handle),
                None,
            )?;

            self.views.drain(..).for_each(|view| {
                self.context.device.destroy_image_view(view, None);
            });

            self.images.clear();

            self.context
                .swapchain_extension
                .destroy_swapchain(self.handle, None);

            self.handle = new_swapchain;
            self.images = self
                .context
                .swapchain_extension
                .get_swapchain_images(self.handle)?;

            for image in &self.images {
                let image_view = self.context.create_image_view(
                    *image,
                    self.format,
                    vk::ImageAspectFlags::COLOR,
                )?;
                self.views.push(image_view);
            }
        }

        Ok(())
    }

    pub fn aquire_next_image(&mut self, semaphore: vk::Semaphore) -> Result<u32> {
        let (image_index, _is_suboptimal) = unsafe {
            self.context.swapchain_extension.acquire_next_image(
                self.handle,
                u64::MAX,
                semaphore,
                vk::Fence::null(),
            )?
        };

        if _is_suboptimal {
            self.is_dirty = true;
        }
        Ok(image_index)
    }
    pub fn present(
        &mut self,
        image_index: u32,
        render_finished_semaphore: &vk::Semaphore,
    ) -> Result<()> {
        let is_suboptimal = unsafe {
            self.context.swapchain_extension.queue_present(
                self.context.queues[self.context.queue_families.present as usize],
                &vk::PresentInfoKHR::default()
                    .wait_semaphores(std::slice::from_ref(render_finished_semaphore))
                    .swapchains(std::slice::from_ref(&self.handle))
                    .image_indices(std::slice::from_ref(&image_index)),
            )?
        };
        if is_suboptimal {
            self.is_dirty = true;
        }
        Ok(())
    }
}

impl Drop for Swapchain {
    fn drop(&mut self) {
        unsafe {
            self.views.drain(..).for_each(|view| {
                self.context.device.destroy_image_view(view, None);
            });
            self.context
                .surface_extension
                .destroy_surface(self.surface.handle, None);
            self.context
                .swapchain_extension
                .destroy_swapchain(self.handle, None);
        }
    }
}
