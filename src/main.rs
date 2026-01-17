use apostasy::engine::{
    ecs::{
        World,
        components::{camera::Camera, transform::Transform},
    },
    rendering::start_renderer,
};
use apostasy_macros::Resource;
use cgmath::{Deg, Quaternion, Rotation3, Vector3};

#[derive(Resource)]
pub struct MyResource {
    pub value: i32,
}

fn main() {
    let world = World::new();

    // Rotate camera to look down at cube from an angle
    let rotation = Quaternion::from_axis_angle(Vector3::new(1.0, 0.0, 0.0), Deg(35.0));

    world.spawn().insert(Camera::default()).insert(Transform {
        position: Vector3::new(0.0, -2.0, 2.0),
        rotation,
        ..Default::default()
    });

    // resource examples
    world.insert_resource::<MyResource>(MyResource { value: 42 });
    world
        .query()
        .include::<Transform>()
        .build()
        .run_with_resources(|entity, mantle| {
            let resources = mantle.resources.read();
            if let Some(my_resource) = resources.get::<MyResource>() {
                println!("Time: {}, ", my_resource.value,);
            }
        });
    world.with_resource::<MyResource, _>(|time| {
        println!("Delta: {}", time.value);
    });
    world.with_resource_mut::<MyResource, _>(|time| {
        time.value += 1;
        println!("Delta: {}", time.value);
    });
    start_renderer(world).unwrap();
}
