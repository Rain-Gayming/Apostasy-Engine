use std::any::Any;

pub trait Component: ComponentContainer {
    fn name() -> &'static str
    where
        Self: Sized;
    fn type_name(&self) -> &'static str;
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

pub trait ComponentContainer {
    fn clone_box(&self) -> Box<dyn Component>;
}

impl<T> ComponentContainer for T
where
    T: 'static + Component + Clone,
{
    fn clone_box(&self) -> Box<dyn Component> {
        Box::new(self.clone())
    }
}
impl Clone for Box<dyn Component> {
    fn clone(&self) -> Box<dyn Component> {
        self.clone_box()
    }
}

pub struct ComponentRegistration {
    pub type_name: &'static str,
    // pub serialize: fn(&dyn Component) -> serde_yaml::Value,
    // pub deserialize: fn(serde_yaml::Value) -> Box<dyn Component>,
    pub create: fn() -> Box<dyn Component>,
}

inventory::collect!(ComponentRegistration);

pub fn get_component_registration(type_name: &str) -> Option<&'static ComponentRegistration> {
    inventory::iter::<ComponentRegistration>()
        .find(|r| r.type_name.to_lowercase() == type_name.to_lowercase())
}
