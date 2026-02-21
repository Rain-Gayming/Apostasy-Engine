use std::{collections::BTreeMap, sync::Arc};

use ash::vk::DescriptorSet;
use egui::{Context, FontFamily};
use egui_ash_renderer::{DynamicRendering, Options};
use winit::window::Window;

use crate::engine::{ecs::World, rendering::swapchain::Swapchain};

pub struct UpdateSystem {
    pub name: &'static str,
    pub func: fn(&mut World),
    pub priority: u32,
}

inventory::collect!(UpdateSystem);

pub struct StartSystem {
    pub name: &'static str,
    pub func: fn(&mut World),
    pub priority: u32,
}

inventory::collect!(StartSystem);

pub struct FixedUpdateSystem {
    pub name: &'static str,
    pub func: fn(&mut World, delta: f32),
    pub priority: u32,
}
inventory::collect!(FixedUpdateSystem);

pub struct LateUpdateSystem {
    pub name: &'static str,
    pub func: fn(&mut World),
    pub priority: u32,
}
inventory::collect!(LateUpdateSystem);

pub struct EguiRenderer {
    pub egui_state: egui_winit::State,
    pub egui_renderer: egui_ash_renderer::Renderer,
    pub egui_ctx: egui::Context,
    pub sorted_ui_systems: Vec<&'static UIFunction>,
}

impl EguiRenderer {
    pub fn new(
        context: &crate::engine::rendering::rendering_context::RenderingContext,
        swapchain: &Swapchain,
        window: &Window,
    ) -> Self {
        let egui_state = egui_winit::State::new(
            egui::Context::default(),
            egui::ViewportId::ROOT,
            &window,
            None,
            None,
            None,
        );

        let mut egui_renderer = egui_ash_renderer::Renderer::with_default_allocator(
            &context.instance,
            context.physical_device.handle,
            context.device.clone(),
            DynamicRendering {
                color_attachment_format: swapchain.format,
                depth_attachment_format: Some(swapchain.depth_format),
            },
            Options::default(),
        )
        .unwrap();
        egui_renderer.add_user_texture(DescriptorSet::default());

        let mut fonts = egui::FontDefinitions::default();
        let mut new_font_family = BTreeMap::new();
        new_font_family.insert(
            FontFamily::Name("fantasy".into()),
            vec!["fantasy".to_owned()],
        );
        fonts.families.append(&mut new_font_family);

        fonts.font_data.insert(
            "fantasy".to_owned(),
            Arc::new(egui::FontData::from_static(include_bytes!(
                "../../../res/fonts/FantasyFont.ttf"
            ))),
        );

        let egui_ctx = egui::Context::default();
        egui_ctx.set_fonts(fonts);

        let mut sorted_ui_systems: Vec<&'static UIFunction> =
            inventory::iter::<UIFunction>.into_iter().collect();
        sorted_ui_systems.sort_by_key(|s| s.priority);
        sorted_ui_systems.reverse();
        Self {
            egui_state,
            egui_renderer,
            egui_ctx,
            sorted_ui_systems,
        }
    }

    pub fn prepare_egui(&mut self, window: &Window, world: &mut World) {
        // Collect input for egui
        let raw_input = self.egui_state.take_egui_input(window);
        self.egui_ctx.begin_pass(raw_input);

        let mut systems: Vec<&UIFunction> = inventory::iter::<UIFunction>.into_iter().collect();
        systems.sort_by_key(|s| s.priority);
        systems.reverse();

        for system in systems {
            (system.func)(&mut self.egui_ctx, world);
        }
    }
}

pub struct UIFunction {
    pub name: &'static str,
    pub func: fn(&mut Context, &mut World),
    pub priority: u32,
}
inventory::collect!(UIFunction);
