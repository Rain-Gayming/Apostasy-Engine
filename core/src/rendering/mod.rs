use std::sync::{Arc, Mutex};

use anyhow::Result;
use winit::{event_loop::ActiveEventLoop, window::Window};

use crate::rendering::{
    shared::rendering_settings::RenderingSettings,
    vulkan::{
        VulkanRenderer,
        queue_family::queue_family_picker,
        rendering_context::{RenderingContextAttributes, VulkanRenderingContext},
    },
};

pub mod opengl;
pub mod shared;
pub mod vulkan;

#[derive(Clone, Copy)]
pub enum RenderingAPIEnum {
    Vulkan,
}

pub struct RenderingInfo {
    /// TODO: change this to a basic rendering context
    pub context: VulkanRenderingContext,
    pub window: Arc<Window>,
    pub settings: RenderingSettings,
    pub renderer: Option<Box<dyn RenderingAPI>>,
}

/// A trait assigned to any Rendering API
/// Used for Vulkan and Opengl
pub trait RenderingAPI {
    fn resize(&mut self) -> Result<()>;
    fn render(&mut self) -> Result<()>;
    fn update_command_buffer(&mut self);
    fn recreate_swapchain(&mut self);
    /// Assigns the rendering_info's renderer the the value created via this
    fn new(rendering_info: Arc<Mutex<RenderingInfo>>) -> Result<()>
    where
        Self: Sized;
}

impl RenderingInfo {
    pub fn new(event_loop: &ActiveEventLoop, rendering_api: RenderingAPIEnum) -> Arc<Mutex<Self>> {
        let window = Arc::new(event_loop.create_window(Default::default()).unwrap());

        let rendering_info = Arc::new(Mutex::new(RenderingInfo {
            context: VulkanRenderingContext::new(RenderingContextAttributes {
                compatability_window: &window,
                queue_family_picker: queue_family_picker::single_queue_family,
            })
            .unwrap(),
            window,
            settings: RenderingSettings::default(),
            renderer: None,
        }));

        match rendering_api {
            RenderingAPIEnum::Vulkan => {
                VulkanRenderer::new(rendering_info.clone()).unwrap();
            }
        }

        rendering_info
    }
}
