use anyhow::Result;
use apostasy_core::log;
use apostasy_macros::{Resource, update};
use cgmath::Vector3;

use crate::objects::{components::transform::Transform, tags::Player, world::World};

#[derive(Resource, Clone)]
pub struct ChunkLoader {
    pub loaded_chunk_ids: Vec<u64>,
    pub last_chunk_position: Vector3<i32>,
}

impl Default for ChunkLoader {
    fn default() -> Self {
        Self {
            loaded_chunk_ids: Vec::new(),
            last_chunk_position: Vector3::new(-1, 0, 0),
        }
    }
}

#[update]
pub fn update_chunks(world: &mut World) -> Result<()> {
    let player = world.get_object_with_tag::<Player>()?;
    let player_transform = player.get_component::<Transform>()?;

    let player_chunk_position = Vector3::new(
        player_transform.global_position.x as i32,
        player_transform.global_position.y as i32,
        player_transform.global_position.z as i32,
    );

    let chunk_loader = world.get_resource_mut::<ChunkLoader>()?;

    if chunk_loader.last_chunk_position != player_chunk_position {
        log!("Entered new chunk");
        chunk_loader.last_chunk_position = player_chunk_position;
    }

    Ok(())
}
