use downcast_rs::Downcast;

use crate::app::engine::ecs::{
    bundle::Bundle,
    components::{position::PositionComponent, velocity::VelocityComponent},
};

pub struct TestBundle {}
impl Bundle for TestBundle {
    fn get_bundle_components() -> Vec<Box<dyn super::component::Component>> {
        vec![
            Box::new(PositionComponent::default()),
            Box::new(VelocityComponent::default()),
        ]
    }
}
