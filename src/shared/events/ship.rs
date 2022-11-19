use std::fmt::Debug;

use bevy::prelude::{Entity, Transform};
use bevy_rapier3d::prelude::Velocity;
use serde::{Deserialize, Serialize};
use spaccegame_proc_macros::NetworkEvent;

use crate::{
    model::{
        block::BlockType,
        block_map::{BlockMap, BlockPosition, BlockRotation},
    },
    shared::remote_refs::{TransformDef, VelocityDef},
};

#[derive(Debug, Serialize, Deserialize, NetworkEvent)]
pub struct SyncShipPositionEvent {
    #[networkEntity]
    #[dropIfNone]
    pub ship_entity: Entity,
    #[serde(with = "TransformDef")]
    pub transform: Transform,
    #[serde(with = "VelocityDef")]
    pub velocity: Velocity,
}

#[derive(Debug, Serialize, Deserialize, NetworkEvent)]
pub struct SyncShipBlocksEvent {
    #[networkEntity]
    #[dropIfNone]
    pub ship_entity: Entity,
    pub block_map: BlockMap,
}

#[derive(Debug, Serialize, Deserialize, NetworkEvent)]
pub struct SyncShipEvent {
    #[networkEntity]
    #[newIfNone]
    pub ship_entity: Entity,
    pub block_map: BlockMap,
    #[serde(with = "TransformDef")]
    pub transform: Transform,
    #[serde(with = "VelocityDef")]
    pub velocity: Velocity,
}

#[derive(Debug, Serialize, Deserialize, NetworkEvent)]
pub struct BlockUpdateEvent {
    #[networkEntity]
    #[dropIfNone]
    pub ship_entity: Entity,
    pub block_type: BlockType,
    pub block_position: BlockPosition,
    pub block_rotation: BlockRotation,
}

#[derive(Debug, Serialize, Deserialize, NetworkEvent)]
pub struct BlockRemoveEvent {
    #[networkEntity]
    #[dropIfNone]
    pub ship_entity: Entity,
    pub block_position: BlockPosition,
}

#[derive(Debug, Serialize, Deserialize, NetworkEvent)]
pub struct EnterShipEvent {
    #[networkEntity]
    #[dropIfNone]
    pub ship_entity: Entity,
}
