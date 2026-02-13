use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::engine::voxels::VoxelTypeId;

/// Base trait for voxel components
pub trait VoxelComponent: Send + Sync + std::fmt::Debug {
    fn as_any_ref(&self) -> &dyn std::any::Any;
    fn clone_arc(&self) -> Arc<dyn VoxelComponent>;
}

impl<T> VoxelComponent for T
where
    T: Send + Sync + std::fmt::Debug + Clone + 'static,
{
    fn as_any_ref(&self) -> &dyn std::any::Any {
        self
    }

    fn clone_arc(&self) -> Arc<dyn VoxelComponent> {
        Arc::new(self.clone())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transitionable {
    pub transitions: Vec<VoxelTransition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoxelTransition {
    pub from: VoxelTypeId,
    pub to: VoxelTypeId,
}

/// Container for all possible voxel components
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VoxelComponents {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transitionable: Option<Transitionable>,
    //
    // #[serde(skip_serializing_if = "Option::is_none")]
    // pub gravity: Option<Gravity>,
    //
    // #[serde(skip_serializing_if = "Option::is_none")]
    // pub flammable: Option<Flammable>,
    //
    // #[serde(skip_serializing_if = "Option::is_none")]
    // pub liquid: Option<Liquid>,
    //
    // #[serde(skip_serializing_if = "Option::is_none")]
    // pub light_emitter: Option<LightEmitter>,
}
