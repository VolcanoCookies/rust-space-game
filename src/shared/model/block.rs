use bevy::prelude::{Bundle, Component, PbrBundle};
use bevy::utils::tracing::Id;
use bevy_rapier3d::geometry::Collider;
use bevy_rapier3d::prelude::{Ccd, Sleeping};
use serde::{Deserialize, Serialize};

use crate::model::block_map::BlockPosition;

#[derive(Bundle)]
pub struct BlockBundle {
    // The type of this block
    pub block_type: BlockType,
    // The position of this block relative to the center of the ship
    pub block_position: BlockPosition,
    #[bundle]
    pub pbr_bundle: PbrBundle,
    pub collider: Collider,
    pub sleeping: Sleeping,
    pub ccd: Ccd,
}

impl Default for BlockBundle {
    fn default() -> Self {
        Self {
            block_type: BlockType::Hull,
            block_position: BlockPosition::default(),
            pbr_bundle: PbrBundle::default(),
            collider: Collider::cuboid(0.5, 0.5, 0.5),
            sleeping: Sleeping::disabled(),
            ccd: Ccd::enabled(),
        }
    }
}

#[derive(Component, Eq, PartialEq, Hash, Clone, Copy, Serialize, Deserialize, Debug)]
pub enum BlockType {
    Hull,
}
