use std::fmt::Debug;

use bevy::prelude::Transform;
use bevy_rapier3d::prelude::Velocity;
use serde::{Deserialize, Serialize};

use crate::{
    model::{
        block::BlockType,
        block_map::{BlockMap, BlockPosition},
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
pub struct AddBlockEvent {
    pub ship_network_id: NetworkId,
    pub block_position: BlockPosition,
    pub block_type: BlockType,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RemoveBlockEvent {
    pub ship_network_id: NetworkId,
    pub block_position: BlockPosition,
}
