use bevy::prelude::{default, Bundle, Component, PbrBundle, Transform};
use bevy_rapier3d::geometry::Collider;
use bevy_rapier3d::prelude::{Ccd, Sleeping};
use serde::{Deserialize, Serialize};

use crate::model::block_map::BlockPosition;

use super::block_map::BlockRotation;

#[derive(Bundle)]
pub struct BlockBundle {
    // The type of this block
    pub block_type: BlockType,
    // The position of this block relative to the center of the ship
    pub block_position: BlockPosition,
    pub block_rotation: BlockRotation,
    #[bundle]
    pub pbr_bundle: PbrBundle,
    pub collider: Collider,
    pub sleeping: Sleeping,
    pub ccd: Ccd,
}

impl BlockBundle {
    pub fn new(
        block_type: BlockType,
        block_position: BlockPosition,
        block_rotation: BlockRotation,
    ) -> Self {
        Self {
            block_type,
            block_position,
            block_rotation,
            pbr_bundle: PbrBundle {
                transform: Transform {
                    translation: block_position.into(),
                    rotation: block_rotation.into(),
                    ..default()
                },
                ..default()
            },
            ..default()
        }
    }

    pub fn with_type(
        block_type: BlockType,
        block_position: BlockPosition,
        block_rotation: BlockRotation,
    ) -> Self {
        Self {
            block_type,
            block_position,
            block_rotation,
            pbr_bundle: PbrBundle {
                transform: Transform {
                    translation: block_position.into(),
                    rotation: block_rotation.into(),
                    ..default()
                },
                ..default()
            },
            ..default()
        }
    }
}

impl Default for BlockBundle {
    fn default() -> Self {
        Self {
            block_type: BlockType::Hull,
            block_position: BlockPosition::default(),
            block_rotation: BlockRotation::default(),
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
