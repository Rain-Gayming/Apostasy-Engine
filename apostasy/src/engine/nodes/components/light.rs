use crate::{self as apostasy};
use apostasy::engine::editor::inspectable::Inspectable;
use apostasy_macros::{Component, InspectValue, Inspectable, SerializableComponent};
use serde::{Deserialize, Serialize};

#[derive(Default, InspectValue, PartialEq, Eq, Clone, Copy, Serialize, Deserialize, Debug)]
pub enum LightType {
    #[default]
    Directional,
    Spot,
    Point,
}

impl Inspectable for LightType {
    fn inspect(
        &mut self,
        ui: &mut egui::Ui,
        editor_storage: &mut crate::engine::editor::EditorStorage,
    ) -> bool {
        egui::ComboBox::from_label("")
            .selected_text(format!("{:?}", self.clone()))
            .show_ui(ui, |ui| {
                ui.selectable_value(self, LightType::Directional, "Directional");
                ui.selectable_value(self, LightType::Spot, "Spot");
                ui.selectable_value(self, LightType::Point, "Point");
            });

        false
    }
}

#[derive(
    Component, Clone, Serialize, Deserialize, SerializableComponent, Inspectable, InspectValue,
)]
pub struct Light {
    pub strength: f32,
    pub light_type: LightType,
}

impl Default for Light {
    fn default() -> Self {
        Self {
            strength: 1.0,
            light_type: LightType::default(),
        }
    }
}
