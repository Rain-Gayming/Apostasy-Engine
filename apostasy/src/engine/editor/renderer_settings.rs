use std::fs::write;

use ash::vk;
use egui::Ui;

use crate::engine::editor::EditorStorage;

pub fn render_renderer_settings(ui: &mut Ui, editor_storage: &mut EditorStorage) {
    let before = editor_storage.pipeline_settings.clone();
    ui.separator();
    ui.label("Depth Settings");
    ui.add(egui::Checkbox::new(
        &mut editor_storage
            .pipeline_settings
            .depth_settings
            .depth_test_enabled,
        "Depth Testing Enabled",
    ));

    egui::ComboBox::from_label("Compare Operation:")
        .selected_text(format!(
            "{:?}",
            editor_storage
                .pipeline_settings
                .depth_settings
                .depth_compare_op
        ))
        .show_ui(ui, |ui| {
            ui.selectable_value(
                &mut editor_storage
                    .pipeline_settings
                    .depth_settings
                    .depth_compare_op,
                vk::CompareOp::NEVER,
                "Never",
            );
            ui.selectable_value(
                &mut editor_storage
                    .pipeline_settings
                    .depth_settings
                    .depth_compare_op,
                vk::CompareOp::LESS,
                "Less",
            );
            ui.selectable_value(
                &mut editor_storage
                    .pipeline_settings
                    .depth_settings
                    .depth_compare_op,
                vk::CompareOp::EQUAL,
                "EQUAL",
            );
            ui.selectable_value(
                &mut editor_storage
                    .pipeline_settings
                    .depth_settings
                    .depth_compare_op,
                vk::CompareOp::LESS_OR_EQUAL,
                "Less or Equal",
            );
            ui.selectable_value(
                &mut editor_storage
                    .pipeline_settings
                    .depth_settings
                    .depth_compare_op,
                vk::CompareOp::GREATER,
                "Greater",
            );
            ui.selectable_value(
                &mut editor_storage
                    .pipeline_settings
                    .depth_settings
                    .depth_compare_op,
                vk::CompareOp::NOT_EQUAL,
                "Not Equal",
            );
            ui.selectable_value(
                &mut editor_storage
                    .pipeline_settings
                    .depth_settings
                    .depth_compare_op,
                vk::CompareOp::GREATER_OR_EQUAL,
                "Greater or Equal",
            );
            ui.selectable_value(
                &mut editor_storage
                    .pipeline_settings
                    .depth_settings
                    .depth_compare_op,
                vk::CompareOp::ALWAYS,
                "Always",
            );
        });

    ui.separator();
    ui.label("Rasterization Settings");

    egui::ComboBox::from_label("Polygon Mode:")
        .selected_text(format!(
            "{:?}",
            editor_storage
                .pipeline_settings
                .rasterization_settings
                .polygon_mode
        ))
        .show_ui(ui, |ui| {
            ui.selectable_value(
                &mut editor_storage
                    .pipeline_settings
                    .rasterization_settings
                    .polygon_mode,
                vk::PolygonMode::FILL,
                "Fill",
            );
            ui.selectable_value(
                &mut editor_storage
                    .pipeline_settings
                    .rasterization_settings
                    .polygon_mode,
                vk::PolygonMode::POINT,
                "Point",
            );
            ui.selectable_value(
                &mut editor_storage
                    .pipeline_settings
                    .rasterization_settings
                    .polygon_mode,
                vk::PolygonMode::LINE,
                "Lines",
            );
        });

    egui::ComboBox::from_label("Cull Mode:")
        .selected_text(format!(
            "{:?}",
            editor_storage
                .pipeline_settings
                .rasterization_settings
                .cull_mode
        ))
        .show_ui(ui, |ui| {
            ui.selectable_value(
                &mut editor_storage
                    .pipeline_settings
                    .rasterization_settings
                    .cull_mode,
                vk::CullModeFlags::NONE,
                "None",
            );
            ui.selectable_value(
                &mut editor_storage
                    .pipeline_settings
                    .rasterization_settings
                    .cull_mode,
                vk::CullModeFlags::FRONT,
                "Front",
            );
            ui.selectable_value(
                &mut editor_storage
                    .pipeline_settings
                    .rasterization_settings
                    .cull_mode,
                vk::CullModeFlags::BACK,
                "Back",
            );
            ui.selectable_value(
                &mut editor_storage
                    .pipeline_settings
                    .rasterization_settings
                    .cull_mode,
                vk::CullModeFlags::FRONT_AND_BACK,
                "Front and Back",
            );
        });

    egui::ComboBox::from_label("Front Face:")
        .selected_text(format!(
            "{:?}",
            editor_storage
                .pipeline_settings
                .rasterization_settings
                .front_face
        ))
        .show_ui(ui, |ui| {
            ui.selectable_value(
                &mut editor_storage
                    .pipeline_settings
                    .rasterization_settings
                    .front_face,
                vk::FrontFace::COUNTER_CLOCKWISE,
                "Counter Clockwise",
            );
            ui.selectable_value(
                &mut editor_storage
                    .pipeline_settings
                    .rasterization_settings
                    .front_face,
                vk::FrontFace::CLOCKWISE,
                "Clockwise",
            );
        });

    ui.add(
        egui::Slider::new(
            &mut editor_storage
                .pipeline_settings
                .rasterization_settings
                .line_width,
            1.0 as f32..=15.0 as f32,
        )
        .show_value(true)
        .text("Line Width"),
    );

    ui.separator();

    ui.label("Image Settings");

    egui::ComboBox::from_label("Filter Mode:")
        .selected_text(format!(
            "{:?}",
            editor_storage.pipeline_settings.image_settings.filter_mode
        ))
        .show_ui(ui, |ui| {
            ui.selectable_value(
                &mut editor_storage.pipeline_settings.image_settings.filter_mode,
                vk::Filter::NEAREST,
                "NEAREST",
            );
            ui.selectable_value(
                &mut editor_storage.pipeline_settings.image_settings.filter_mode,
                vk::Filter::LINEAR,
                "LINEAR",
            );
        });

    egui::ComboBox::from_label("Address Mode:")
        .selected_text(format!(
            "{:?}",
            editor_storage.pipeline_settings.image_settings.address_mode
        ))
        .show_ui(ui, |ui| {
            ui.selectable_value(
                &mut editor_storage.pipeline_settings.image_settings.address_mode,
                vk::SamplerAddressMode::REPEAT,
                "Repeat",
            );
            ui.selectable_value(
                &mut editor_storage.pipeline_settings.image_settings.address_mode,
                vk::SamplerAddressMode::MIRRORED_REPEAT,
                "Mirrored Repeat",
            );
            ui.selectable_value(
                &mut editor_storage.pipeline_settings.image_settings.address_mode,
                vk::SamplerAddressMode::CLAMP_TO_EDGE,
                "Clamp to Edge",
            );
            ui.selectable_value(
                &mut editor_storage.pipeline_settings.image_settings.address_mode,
                vk::SamplerAddressMode::CLAMP_TO_BORDER,
                "Clamp to Border",
            );
        });

    egui::ComboBox::from_label("Mipmap Mode:")
        .selected_text(format!(
            "{:?}",
            editor_storage.pipeline_settings.image_settings.mip_map_mode
        ))
        .show_ui(ui, |ui| {
            ui.selectable_value(
                &mut editor_storage.pipeline_settings.image_settings.mip_map_mode,
                vk::SamplerMipmapMode::NEAREST,
                "NEAREST",
            );
            ui.selectable_value(
                &mut editor_storage.pipeline_settings.image_settings.mip_map_mode,
                vk::SamplerMipmapMode::LINEAR,
                "LINEAR",
            );
        });

    ui.add(egui::Checkbox::new(
        &mut editor_storage
            .pipeline_settings
            .image_settings
            .anisotropy_enabled,
        "Anisotropy Enabled",
    ));

    if editor_storage
        .pipeline_settings
        .image_settings
        .anisotropy_enabled
    {
        egui::ComboBox::from_label("Anisotropy Amount")
            .selected_text(format!(
                "{:?}",
                editor_storage
                    .pipeline_settings
                    .image_settings
                    .anisotropy_amount
            ))
            .show_ui(ui, |ui| {
                ui.selectable_value(
                    &mut editor_storage
                        .pipeline_settings
                        .image_settings
                        .anisotropy_amount,
                    2,
                    "2x",
                );
                ui.selectable_value(
                    &mut editor_storage
                        .pipeline_settings
                        .image_settings
                        .anisotropy_amount,
                    4,
                    "4x",
                );
                ui.selectable_value(
                    &mut editor_storage
                        .pipeline_settings
                        .image_settings
                        .anisotropy_amount,
                    8,
                    "8x",
                );
                ui.selectable_value(
                    &mut editor_storage
                        .pipeline_settings
                        .image_settings
                        .anisotropy_amount,
                    16,
                    "16x",
                );
            });
    }

    ui.separator();
    ui.label("Debug Settings");

    ui.add(
        egui::Slider::new(
            &mut editor_storage
                .pipeline_settings
                .debug_settings
                .debug_line_width,
            1.0 as f32..=15.0 as f32,
        )
        .show_value(true)
        .text("Debug Line Width"),
    );

    ui.add(egui::Checkbox::new(
        &mut editor_storage
            .pipeline_settings
            .debug_settings
            .collision_debug_enabled,
        "Collision Debug Outlines Enabled",
    ));

    if editor_storage.pipeline_settings != before {
        editor_storage.should_update_renderer = true;

        if let Ok(s) = serde_yaml::to_string(&editor_storage.pipeline_settings) {
            let _ = write("res/.engine/pipeline_settings.yaml", s);
        }
    }
}
