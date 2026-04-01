use cgmath::{Quaternion, Vector2, Vector3};
use egui::{DragValue, Ui};

use crate::engine::editor::EditorStorage;

/// Implemented by structs that can be inspected
/// Impliment via ```#[derive(Inspectable)]```
pub trait Inspectable {
    fn inspect(&mut self, ui: &mut Ui, editor_storage: &mut EditorStorage) -> bool;
    fn on_inspect(&mut self) {}
}

/// Implemented by types that can be inspected
/// for structs this can be done automatically via `#[derive(InspectValue)]`
/// but you can implement it manually if you want to add custom functionality
/// Impliment via
/// ```
/// impl InspectValue for YourType {
///     fn inspect_value(&mut self, ui: &mut egui::Ui) {
///         // egui values needed
///         //ui.add(egui::DragValue::new(self).speed(0.1));
///
///         // Custom functions called here
///     }
/// }
///     
/// }```
pub trait InspectValue {
    fn inspect_value(&mut self, ui: &mut Ui, _editor_storage: &mut EditorStorage);
}

impl InspectValue for f32 {
    fn inspect_value(&mut self, ui: &mut Ui, __editor_storage: &mut EditorStorage) {
        let mut value = if self.is_finite() { *self as f64 } else { 0.0 };
        ui.add(DragValue::new(&mut value).speed(0.01));
        *self = value as f32;
    }
}

impl InspectValue for f64 {
    fn inspect_value(&mut self, ui: &mut Ui, _editor_storage: &mut EditorStorage) {
        let mut value = if self.is_finite() { *self } else { 0.0 };
        ui.add(DragValue::new(&mut value).speed(0.01));
        *self = value;
    }
}

impl InspectValue for i32 {
    fn inspect_value(&mut self, ui: &mut Ui, _editor_storage: &mut EditorStorage) {
        ui.add(egui::DragValue::new(self));
    }
}

impl InspectValue for u32 {
    fn inspect_value(&mut self, ui: &mut Ui, _editor_storage: &mut EditorStorage) {
        ui.add(egui::DragValue::new(self));
    }
}
impl InspectValue for u8 {
    fn inspect_value(&mut self, ui: &mut Ui, _editor_storage: &mut EditorStorage) {
        ui.add(egui::DragValue::new(self));
    }
}

impl InspectValue for u64 {
    fn inspect_value(&mut self, ui: &mut Ui, _editor_storage: &mut EditorStorage) {
        ui.add(egui::DragValue::new(self));
    }
}

impl InspectValue for String {
    fn inspect_value(&mut self, ui: &mut Ui, _editor_storage: &mut EditorStorage) {
        ui.text_edit_singleline(self);
    }
}

impl InspectValue for bool {
    fn inspect_value(&mut self, ui: &mut Ui, _editor_storage: &mut EditorStorage) {
        ui.checkbox(self, "");
    }
}
impl InspectValue for [f32; 3] {
    fn inspect_value(&mut self, ui: &mut Ui, _editor_storage: &mut EditorStorage) {
        let mut x = if self[0].is_finite() {
            self[0] as f64
        } else {
            0.0
        };
        let mut y = if self[1].is_finite() {
            self[1] as f64
        } else {
            0.0
        };
        let mut z = if self[2].is_finite() {
            self[2] as f64
        } else {
            0.0
        };

        ui.add(DragValue::new(&mut x).speed(0.01));
        ui.add(DragValue::new(&mut y).speed(0.01));
        ui.add(DragValue::new(&mut z).speed(0.01));

        self[0] = x as f32;
        self[1] = y as f32;
        self[2] = z as f32;
    }
}

impl InspectValue for [f32; 2] {
    fn inspect_value(&mut self, ui: &mut Ui, _editor_storage: &mut EditorStorage) {
        let mut x = if self[0].is_finite() {
            self[0] as f64
        } else {
            0.0
        };
        let mut y = if self[1].is_finite() {
            self[1] as f64
        } else {
            0.0
        };

        ui.add(DragValue::new(&mut x).speed(0.01));
        ui.add(DragValue::new(&mut y).speed(0.01));

        self[0] = x as f32;
        self[1] = y as f32;
    }
}

impl InspectValue for Vector3<f32> {
    fn inspect_value(&mut self, ui: &mut Ui, _editor_storage: &mut EditorStorage) {
        let mut x = if self.x.is_finite() {
            self.x as f64
        } else {
            0.0
        };
        let mut y = if self.y.is_finite() {
            self.y as f64
        } else {
            0.0
        };
        let mut z = if self.z.is_finite() {
            self.z as f64
        } else {
            0.0
        };

        ui.add(DragValue::new(&mut x).speed(0.01));
        ui.add(DragValue::new(&mut y).speed(0.01));
        ui.add(DragValue::new(&mut z).speed(0.01));

        self.x = x as f32;
        self.y = y as f32;
        self.z = z as f32;
    }
}

impl InspectValue for Quaternion<f32> {
    fn inspect_value(&mut self, ui: &mut Ui, _editor_storage: &mut EditorStorage) {
        let mut s = if self.s.is_finite() {
            self.s as f64
        } else {
            0.0
        };
        let mut x = if self.v.x.is_finite() {
            self.v.x as f64
        } else {
            0.0
        };
        let mut y = if self.v.y.is_finite() {
            self.v.y as f64
        } else {
            0.0
        };
        let mut z = if self.v.z.is_finite() {
            self.v.z as f64
        } else {
            0.0
        };

        ui.add(DragValue::new(&mut s).speed(0.01));
        ui.add(DragValue::new(&mut x).speed(0.01));
        ui.add(DragValue::new(&mut y).speed(0.01));
        ui.add(DragValue::new(&mut z).speed(0.01));

        self.s = s as f32;
        self.v.x = x as f32;
        self.v.y = y as f32;
        self.v.z = z as f32;
    }
}
impl InspectValue for Vector3<f64> {
    fn inspect_value(&mut self, ui: &mut Ui, _editor_storage: &mut EditorStorage) {
        let mut x = if self.x.is_finite() { self.x } else { 0.0 };
        let mut y = if self.y.is_finite() { self.y } else { 0.0 };
        let mut z = if self.z.is_finite() { self.z } else { 0.0 };

        ui.add(DragValue::new(&mut x).speed(0.01));
        ui.add(DragValue::new(&mut y).speed(0.01));
        ui.add(DragValue::new(&mut z).speed(0.01));

        self.x = x;
        self.y = y;
        self.z = z;
    }
}

impl InspectValue for Quaternion<f64> {
    fn inspect_value(&mut self, ui: &mut Ui, _editor_storage: &mut EditorStorage) {
        let mut s = if self.s.is_finite() { self.s } else { 0.0 };
        let mut x = if self.v.x.is_finite() { self.v.x } else { 0.0 };
        let mut y = if self.v.y.is_finite() { self.v.y } else { 0.0 };
        let mut z = if self.v.z.is_finite() { self.v.z } else { 0.0 };

        ui.add(DragValue::new(&mut s).speed(0.01));
        ui.add(DragValue::new(&mut x).speed(0.01));
        ui.add(DragValue::new(&mut y).speed(0.01));
        ui.add(DragValue::new(&mut z).speed(0.01));

        self.s = s;
        self.v.x = x;
        self.v.y = y;
        self.v.z = z;
    }
}
impl InspectValue for Vector3<i8> {
    fn inspect_value(&mut self, ui: &mut Ui, _editor_storage: &mut EditorStorage) {
        ui.add(DragValue::new(&mut self.x).speed(1));
        ui.add(DragValue::new(&mut self.y).speed(1));
        ui.add(DragValue::new(&mut self.z).speed(1));
    }
}

impl InspectValue for Quaternion<i8> {
    fn inspect_value(&mut self, ui: &mut Ui, _editor_storage: &mut EditorStorage) {
        ui.add(DragValue::new(&mut self.s).speed(1));
        ui.add(DragValue::new(&mut self.v.x).speed(1));
        ui.add(DragValue::new(&mut self.v.y).speed(1));
        ui.add(DragValue::new(&mut self.v.z).speed(1));
    }
}

impl InspectValue for Vector3<i16> {
    fn inspect_value(&mut self, ui: &mut Ui, _editor_storage: &mut EditorStorage) {
        ui.add(DragValue::new(&mut self.x).speed(1));
        ui.add(DragValue::new(&mut self.y).speed(1));
        ui.add(DragValue::new(&mut self.z).speed(1));
    }
}

impl InspectValue for Quaternion<i16> {
    fn inspect_value(&mut self, ui: &mut Ui, _editor_storage: &mut EditorStorage) {
        ui.add(DragValue::new(&mut self.s).speed(1));
        ui.add(DragValue::new(&mut self.v.x).speed(1));
        ui.add(DragValue::new(&mut self.v.y).speed(1));
        ui.add(DragValue::new(&mut self.v.z).speed(1));
    }
}

impl InspectValue for Quaternion<i32> {
    fn inspect_value(&mut self, ui: &mut Ui, _editor_storage: &mut EditorStorage) {
        ui.add(DragValue::new(&mut self.s).speed(1));
        ui.add(DragValue::new(&mut self.v.x).speed(1));
        ui.add(DragValue::new(&mut self.v.y).speed(1));
        ui.add(DragValue::new(&mut self.v.z).speed(1));
    }
}

impl InspectValue for Vector3<i32> {
    fn inspect_value(&mut self, ui: &mut Ui, _editor_storage: &mut EditorStorage) {
        ui.add(DragValue::new(&mut self.x).speed(1));
        ui.add(DragValue::new(&mut self.y).speed(1));
        ui.add(DragValue::new(&mut self.z).speed(1));
    }
}

impl InspectValue for Vector3<i64> {
    fn inspect_value(&mut self, ui: &mut Ui, _editor_storage: &mut EditorStorage) {
        ui.add(DragValue::new(&mut self.x).speed(1));
        ui.add(DragValue::new(&mut self.y).speed(1));
        ui.add(DragValue::new(&mut self.z).speed(1));
    }
}

impl InspectValue for Vector2<i32> {
    fn inspect_value(&mut self, ui: &mut Ui, _editor_storage: &mut EditorStorage) {
        ui.add(DragValue::new(&mut self.x).speed(1));
        ui.add(DragValue::new(&mut self.y).speed(1));
    }
}

impl InspectValue for Vector2<i64> {
    fn inspect_value(&mut self, ui: &mut Ui, _editor_storage: &mut EditorStorage) {
        ui.add(DragValue::new(&mut self.x).speed(1));
        ui.add(DragValue::new(&mut self.y).speed(1));
    }
}

impl InspectValue for Quaternion<i64> {
    fn inspect_value(&mut self, ui: &mut Ui, _editor_storage: &mut EditorStorage) {
        ui.add(DragValue::new(&mut self.s).speed(1));
        ui.add(DragValue::new(&mut self.v.x).speed(1));
        ui.add(DragValue::new(&mut self.v.y).speed(1));
        ui.add(DragValue::new(&mut self.v.z).speed(1));
    }
}

impl<T: InspectValue> InspectValue for Option<T> {
    fn inspect_value(&mut self, ui: &mut Ui, editor_storage: &mut EditorStorage) {
        match self {
            Some(val) => {
                ui.horizontal(|ui| {
                    ui.label("Some:");
                    val.inspect_value(ui, editor_storage);
                });
            }
            None => {
                ui.label("None");
            }
        }
    }
}

impl<T: InspectValue> InspectValue for Vec<T> {
    fn inspect_value(&mut self, ui: &mut Ui, editor_storage: &mut EditorStorage) {
        egui::CollapsingHeader::new(format!("Vec ({})", self.len())).show(ui, |ui| {
            for (i, item) in self.iter_mut().enumerate() {
                ui.horizontal(|ui| {
                    ui.label(format!("[{}]", i));
                    item.inspect_value(ui, editor_storage);
                });
            }
        });
    }
}
