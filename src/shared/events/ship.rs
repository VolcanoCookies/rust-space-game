use std::fmt::Debug;

use bevy::{
    ecs::entity::Entities,
    prelude::{Entity, Transform},
};
use bevy_rapier3d::prelude::Velocity;
use serde::{Deserialize, Serialize};
use spaccegame_proc_macros::{networkEntity, NetworkEvent};

use crate::{
    model::{
        block::BlockType,
        block_map::{BlockMap, BlockPosition, BlockRotation},
    },
    shared::{
        networking::network_id::NetworkId,
        remote_refs::{TransformDef, VelocityDef},
    },
};

#[derive(Debug, Serialize, Deserialize)]
pub struct SyncShipPositionEvent {
    pub ship_network_id: NetworkId,
    #[serde(with = "TransformDef")]
    pub transform: Transform,
    #[serde(with = "VelocityDef")]
    pub velocity: Velocity,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SyncShipBlocksEvent {
    pub ship_network_id: NetworkId,
    pub block_map: BlockMap,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SyncShipEvent {
    pub ship_network_id: NetworkId,
    pub block_map: BlockMap,
    #[serde(with = "TransformDef")]
    pub transform: Transform,
    #[serde(with = "VelocityDef")]
    pub velocity: Velocity,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PlaceBlockEvent {
    pub ship_network_id: NetworkId,
    pub block_type: BlockType,
    pub block_position: BlockPosition,
    pub block_rotation: BlockRotation,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RemoveBlockEvent {
    pub ship_network_id: NetworkId,
    pub block_position: BlockPosition,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BlockUpdateEvent {
    pub ship_network_id: NetworkId,
    pub block_type: BlockType,
    pub block_position: BlockPosition,
    pub block_rotation: BlockRotation,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BlockRemoveEvent {
    pub ship_network_id: NetworkId,
    pub block_position: BlockPosition,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EnterShipEvent {
    pub ship_network_id: NetworkId,
}

#[derive(NetworkEvent)]
struct Test {
    pub ship_entity: Entity,
    pub test: u32,
    a: i16,
}
