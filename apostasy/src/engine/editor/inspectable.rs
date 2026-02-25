use cgmath::{Matrix, Quaternion, Vector3};
use egui::DragValue;

pub trait Inspectable {
    fn inspect(&mut self, ui: &mut egui::Ui);
}

pub trait InspectValue {
    fn inspect_value(&mut self, ui: &mut egui::Ui);
}

// Implement InspectValue for ALL common types
impl InspectValue for f32 {
    fn inspect_value(&mut self, ui: &mut egui::Ui) {
        ui.add(egui::DragValue::new(self).speed(0.1));
    }
}

impl InspectValue for f64 {
    fn inspect_value(&mut self, ui: &mut egui::Ui) {
        ui.add(egui::DragValue::new(self).speed(0.1));
    }
}

impl InspectValue for i32 {
    fn inspect_value(&mut self, ui: &mut egui::Ui) {
        ui.add(egui::DragValue::new(self));
    }
}

impl InspectValue for u32 {
    fn inspect_value(&mut self, ui: &mut egui::Ui) {
        ui.add(egui::DragValue::new(self));
    }
}

impl InspectValue for String {
    fn inspect_value(&mut self, ui: &mut egui::Ui) {
        ui.text_edit_singleline(self);
    }
}

impl InspectValue for bool {
    fn inspect_value(&mut self, ui: &mut egui::Ui) {
        ui.checkbox(self, "");
    }
}

impl InspectValue for Vector3<f32> {
    fn inspect_value(&mut self, ui: &mut egui::Ui) {
        ui.add(DragValue::new(&mut self.x).speed(1));
        ui.add(DragValue::new(&mut self.y).speed(1));
        ui.add(DragValue::new(&mut self.z).speed(1));
    }
}

impl InspectValue for Quaternion<f32> {
    fn inspect_value(&mut self, ui: &mut egui::Ui) {
        ui.add(DragValue::new(&mut self.s).speed(1));
        ui.add(DragValue::new(&mut self.v.x).speed(1));
        ui.add(DragValue::new(&mut self.v.y).speed(1));
        ui.add(DragValue::new(&mut self.v.z).speed(1));
    }
}
impl InspectValue for Vector3<f64> {
    fn inspect_value(&mut self, ui: &mut egui::Ui) {
        ui.add(DragValue::new(&mut self.x).speed(1));
        ui.add(DragValue::new(&mut self.y).speed(1));
        ui.add(DragValue::new(&mut self.z).speed(1));
    }
}

impl InspectValue for Quaternion<f64> {
    fn inspect_value(&mut self, ui: &mut egui::Ui) {
        ui.add(DragValue::new(&mut self.s).speed(1));
        ui.add(DragValue::new(&mut self.v.x).speed(1));
        ui.add(DragValue::new(&mut self.v.y).speed(1));
        ui.add(DragValue::new(&mut self.v.z).speed(1));
    }
}
impl InspectValue for Vector3<i8> {
    fn inspect_value(&mut self, ui: &mut egui::Ui) {
        ui.add(DragValue::new(&mut self.x).speed(1));
        ui.add(DragValue::new(&mut self.y).speed(1));
        ui.add(DragValue::new(&mut self.z).speed(1));
    }
}

impl InspectValue for Quaternion<i8> {
    fn inspect_value(&mut self, ui: &mut egui::Ui) {
        ui.add(DragValue::new(&mut self.s).speed(1));
        ui.add(DragValue::new(&mut self.v.x).speed(1));
        ui.add(DragValue::new(&mut self.v.y).speed(1));
        ui.add(DragValue::new(&mut self.v.z).speed(1));
    }
}

impl InspectValue for Vector3<i16> {
    fn inspect_value(&mut self, ui: &mut egui::Ui) {
        ui.add(DragValue::new(&mut self.x).speed(1));
        ui.add(DragValue::new(&mut self.y).speed(1));
        ui.add(DragValue::new(&mut self.z).speed(1));
    }
}

impl InspectValue for Quaternion<i16> {
    fn inspect_value(&mut self, ui: &mut egui::Ui) {
        ui.add(DragValue::new(&mut self.s).speed(1));
        ui.add(DragValue::new(&mut self.v.x).speed(1));
        ui.add(DragValue::new(&mut self.v.y).speed(1));
        ui.add(DragValue::new(&mut self.v.z).speed(1));
    }
}

impl InspectValue for Quaternion<i32> {
    fn inspect_value(&mut self, ui: &mut egui::Ui) {
        ui.add(DragValue::new(&mut self.s).speed(1));
        ui.add(DragValue::new(&mut self.v.x).speed(1));
        ui.add(DragValue::new(&mut self.v.y).speed(1));
        ui.add(DragValue::new(&mut self.v.z).speed(1));
    }
}

impl InspectValue for Vector3<i32> {
    fn inspect_value(&mut self, ui: &mut egui::Ui) {
        ui.add(DragValue::new(&mut self.x).speed(1));
        ui.add(DragValue::new(&mut self.y).speed(1));
        ui.add(DragValue::new(&mut self.z).speed(1));
    }
}

impl InspectValue for Vector3<i64> {
    fn inspect_value(&mut self, ui: &mut egui::Ui) {
        ui.add(DragValue::new(&mut self.x).speed(1));
        ui.add(DragValue::new(&mut self.y).speed(1));
        ui.add(DragValue::new(&mut self.z).speed(1));
    }
}

impl InspectValue for Quaternion<i64> {
    fn inspect_value(&mut self, ui: &mut egui::Ui) {
        ui.add(DragValue::new(&mut self.s).speed(1));
        ui.add(DragValue::new(&mut self.v.x).speed(1));
        ui.add(DragValue::new(&mut self.v.y).speed(1));
        ui.add(DragValue::new(&mut self.v.z).speed(1));
    }
}

impl<T: InspectValue> InspectValue for Option<T> {
    fn inspect_value(&mut self, ui: &mut egui::Ui) {
        match self {
            Some(val) => {
                ui.horizontal(|ui| {
                    ui.label("Some:");
                    val.inspect_value(ui);
                });
            }
            None => {
                ui.label("None");
            }
        }
    }
}

impl<T: InspectValue> InspectValue for Vec<T> {
    fn inspect_value(&mut self, ui: &mut egui::Ui) {
        egui::CollapsingHeader::new(format!("Vec ({})", self.len())).show(ui, |ui| {
            for (i, item) in self.iter_mut().enumerate() {
                ui.horizontal(|ui| {
                    ui.label(format!("[{}]", i));
                    item.inspect_value(ui);
                });
            }
        });
    }
}
