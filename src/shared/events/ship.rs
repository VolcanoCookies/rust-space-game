use std::fmt::Debug;

use bevy::prelude::{Entity, Transform};
use bevy_rapier3d::prelude::Velocity;
use serde::{Deserialize, Serialize};
use spacegame_proc_macros::client_bound;

use crate::{
    model::{
        block::BlockType,
        block_map::{BlockMap, BlockPosition, BlockRotation},
    },
    shared::remote_refs::{TransformDef, VelocityDef},
};

#[derive(Debug, Serialize, Deserialize)]
#[client_bound]
pub struct SyncShipPositionEvent {
    #[entity]
    #[missing = "drop"]
    pub ship_entity: Entity,
    #[serde(with = "TransformDef")]
    pub transform: Transform,
    #[serde(with = "VelocityDef")]
    pub velocity: Velocity,
}

#[derive(Debug, Serialize, Deserialize)]
#[client_bound]
pub struct SyncShipBlocksEvent {
    #[entity]
    #[missing = "drop"]
    pub ship_entity: Entity,
    pub block_map: BlockMap,
}

#[derive(Debug, Serialize, Deserialize)]
#[client_bound]
pub struct SyncShipEvent {
    #[entity]
    #[missing = "drop"]
    pub ship_entity: Entity,
    pub block_map: BlockMap,
    #[serde(with = "TransformDef")]
    pub transform: Transform,
    #[serde(with = "VelocityDef")]
    pub velocity: Velocity,
}

#[derive(Debug, Serialize, Deserialize)]
#[client_bound]
pub struct BlockUpdateEvent {
    #[entity]
    #[missing = "drop"]
    pub ship_entity: Entity,
    pub block_type: BlockType,
    pub block_position: BlockPosition,
    pub block_rotation: BlockRotation,
}

#[derive(Debug, Serialize, Deserialize)]
#[client_bound]
pub struct BlockRemoveEvent {
    #[entity]
    #[missing = "drop"]
    pub ship_entity: Entity,
    pub block_position: BlockPosition,
}

#[derive(Debug, Serialize, Deserialize)]
#[client_bound]
pub struct EnterShipEvent {
    #[entity]
    #[missing = "drop"]
    pub ship_entity: Entity,
}
