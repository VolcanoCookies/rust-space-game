use std::fmt::Debug;

use bevy::prelude::{Entity, Transform};
use bevy_rapier3d::prelude::{ExternalForce, Velocity};
use serde::{Deserialize, Serialize};
use spacegame_core::message::ClientId;
use spacegame_proc_macros::{bidirectional, client_bound, server_bound};

use crate::{
    model::{
        block::BlockType,
        block_map::{BlockMap, BlockPosition, BlockRotation},
    },
    shared::remote_refs::{ExternalForceDef, TransformDef, VelocityDef},
};

#[client_bound]
#[derive(Serialize, Deserialize)]
pub struct SyncShipPositionEvent {
    #[entity]
    #[missing = "drop"]
    pub ship_entity: Entity,
    #[serde(with = "TransformDef")]
    pub transform: Transform,
    #[serde(with = "VelocityDef")]
    pub velocity: Velocity,
}

#[client_bound]
#[derive(Serialize, Deserialize)]
pub struct SyncShipBlocksEvent {
    #[entity]
    #[missing = "drop"]
    pub ship_entity: Entity,
    pub block_map: BlockMap,
}

#[client_bound]
#[derive(Serialize, Deserialize)]
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

#[client_bound]
#[derive(Serialize, Deserialize)]
pub struct LoadShipEvent {
    #[entity]
    #[missing = "create"]
    pub ship_entity: Entity,
    pub block_map: BlockMap,
    #[serde(with = "TransformDef")]
    pub transform: Transform,
    #[serde(with = "VelocityDef")]
    pub velocity: Velocity,
    pub name: String,
}

#[client_bound]
#[derive(Serialize, Deserialize)]
pub struct UnloadShipEvent {
    #[entity]
    #[missing = "drop"]
    pub ship_entity: Entity,
}

#[bidirectional]
#[derive(Serialize, Deserialize)]
pub struct BlockUpdateEvent {
    #[entity]
    #[missing = "drop"]
    pub ship_entity: Entity,
    pub block_type: BlockType,
    pub block_position: BlockPosition,
    pub block_rotation: BlockRotation,
}

#[bidirectional]
#[derive(Serialize, Deserialize)]
pub struct BlockRemoveEvent {
    #[entity]
    #[missing = "drop"]
    pub ship_entity: Entity,
    pub block_position: BlockPosition,
}

#[server_bound]
#[derive(Serialize, Deserialize)]
pub struct TryEnterShipEvent {
    #[entity]
    #[missing = "drop"]
    pub ship_entity: Entity,
}

#[client_bound]
#[derive(Serialize, Deserialize)]
pub struct EnteredShipEvent {
    #[entity]
    #[missing = "drop"]
    pub ship_entity: Entity,
    pub player_id: ClientId,
}

#[server_bound]
#[derive(Serialize, Deserialize)]
pub struct TryLeaveShipEvent {
    #[entity]
    #[missing = "drop"]
    pub ship_entity: Entity,
}

#[client_bound]
#[derive(Serialize, Deserialize)]
pub struct LeftShipEvent {
    #[entity]
    #[missing = "drop"]
    pub ship_entity: Entity,
    pub player_id: ClientId,
}

#[server_bound]
#[derive(Serialize, Deserialize)]
pub struct ShipMoveEvent {
    #[entity]
    #[missing = "drop"]
    pub ship_entity: Entity,
    #[serde(with = "ExternalForceDef")]
    pub force: ExternalForce,
}
