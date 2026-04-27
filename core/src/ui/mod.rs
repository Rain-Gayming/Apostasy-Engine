use anyhow::Result;
use ash::vk::DescriptorSet;
use egui::{Context, FontDefinitions};
use egui_ash_renderer::{DynamicRendering, Options, Renderer};
use egui_winit::State;
use winit::window::Window;

use crate::rendering::vulkan::{
    rendering_context::VulkanRenderingContext, swapchain::VulkanSwapchain,
};

pub struct UIRenderer {
    pub state: State,
    pub renderer: Renderer,
    pub context: Context,
}

impl UIRenderer {
    pub fn new(
        context: VulkanRenderingContext,
        swapchain: &VulkanSwapchain,
        window: &Window,
    ) -> Result<Self> {
        let mut renderer = Renderer::with_default_allocator(
            &context.instance,
            context.physical_device.handle,
            context.device.clone(),
            DynamicRendering {
                color_attachment_format: swapchain.format,
                depth_attachment_format: Some(swapchain.depth_format),
            },
            Options {
                srgb_framebuffer: true,
                ..Default::default()
            },
        )?;

        renderer.add_user_texture(DescriptorSet::default());

        let mut fonts = FontDefinitions::default();
        // let mut font_family = BTreeMap::new();

        // TODO: impliment font here

        let context = Context::default();
        context.set_fonts(fonts);
        // TODO: make style
        // context.set_style(style);

        let state = State::new(
            context.clone(),
            egui::ViewportId::ROOT,
            &window,
            None,
            None,
            None,
        );

        Ok(Self {
            state,
            renderer,
            context,
        })
    }
}
