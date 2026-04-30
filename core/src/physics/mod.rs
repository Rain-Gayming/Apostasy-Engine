use apostasy_macros::Component;

pub mod collider;
pub mod collision_system;
pub mod velocity;

#[derive(Component, Clone, Debug)]
pub struct Gravity {
    pub strength: f32,
}

impl Default for Gravity {
    fn default() -> Self {
        Self { strength: 9.81 }
    }
}

impl Gravity {
    pub fn deserialize(&mut self, _value: &serde_yaml::Value) -> anyhow::Result<()> {
        Ok(())
    }
}
