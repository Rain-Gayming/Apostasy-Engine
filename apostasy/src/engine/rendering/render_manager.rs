use std::{collections::HashMap, sync::Arc};

use anyhow::Result;
use ash::{
    khr::surface,
    vk::{self, ApplicationInfo, InstanceCreateInfo},
};
use winit::{
    raw_window_handle::{HasDisplayHandle, HasWindowHandle},
    window::{Window, WindowId},
};

use crate::engine::rendering::renderer::Renderer;

pub struct RenderManager {
    pub surface_extensions: ash::khr::surface::Instance,
    pub instance: ash::Instance,
    pub entry: ash::Entry,
    pub renderers: HashMap<WindowId, Renderer>,
}

impl RenderManager {
    pub fn new(window: &Window) -> Result<Self> {
        unsafe {
            let entry = ash::Entry::load()?;

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

            Ok(Self {
                surface_extensions,
                instance,
                entry,
                renderers: HashMap::new(),
            })
        }
    }
}
