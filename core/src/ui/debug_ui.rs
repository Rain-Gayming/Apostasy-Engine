use anyhow::Result;
use apostasy_macros::update;

use crate::{
    objects::{components::transform::Transform, systems::DeltaTime, tags::Player, world::World},
    rendering::components::camera::ActiveCamera,
    ui::ui_context::EguiContext,
    voxels::chunk::Chunk,
};

#[update]
pub fn hud(world: &mut World) -> Result<()> {
    let ctx = world.get_resource::<EguiContext>()?.0.clone();

    egui::Area::new(egui::Id::new("crosshair"))
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(&ctx, |ui| {
            ui.label(
                egui::RichText::new("+")
                    .size(24.0)
                    .color(egui::Color32::WHITE),
            );
        });

    egui::Window::new("Debug")
        .anchor(egui::Align2::LEFT_TOP, [10.0, 10.0])
        .show(&ctx, |ui| {
            if let Ok(dt) = world.get_resource::<DeltaTime>() {
                ui.label(format!("FPS: {:.0}", 1.0 / dt.0));
            }
            ui.label(format!(
                "Chunks: {}",
                world.get_objects_with_component::<Chunk>().len()
            ));

            if let Ok(player) = world.get_object_with_tag::<Player>() {
                let transform = player.get_component::<Transform>().unwrap();
                ui.label(format!("Position: {:?}", transform.local_position));
                ui.label(format!("Global Position: {:?}", transform.global_position));
            }

            if let Ok(camera) = world.get_object_with_tag::<ActiveCamera>() {
                let transform = camera.get_component::<Transform>().unwrap();
                ui.label(format!("Cam Position: {:?}", transform.local_position));
                ui.label(format!(
                    "Cam Global Position: {:?}",
                    transform.global_position
                ));
            }
        });

    Ok(())
}
