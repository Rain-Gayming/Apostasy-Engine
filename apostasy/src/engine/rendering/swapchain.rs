use std::sync::Arc;

use anyhow::Result;
use ash::vk::{self};
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
}

impl Swapchain {
    pub fn new(context: Arc<RenderingContext>, window: Arc<Window>) -> Result<Self> {
        unsafe {
            let surface = context.create_surface(&window)?;
            let format = vk::Format::B8G8R8A8_SRGB;
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
            })
        }
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
        }

        Ok(())
    }
}
