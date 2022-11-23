use bevy::prelude::{Entity, Transform};
use serde::{Deserialize, Serialize};
use spacegame_core::message::ClientId;
use spacegame_proc_macros::{bidirectional, client_bound};

use crate::shared::remote_refs::TransformDef;

#[bidirectional]
#[derive(Serialize, Deserialize)]
pub struct PlayerMoveEvent {
    #[serde(with = "TransformDef")]
    pub transform: Transform,
}

#[client_bound]
#[derive(Serialize, Deserialize)]
pub struct PlayerSpawnEvent {
    #[entity]
    #[missing = "create"]
    pub player_entity: Entity,
    pub player_name: String,
    #[serde(with = "TransformDef")]
    pub transform: Transform,
    pub player_id: ClientId,
}

#[client_bound]
#[derive(Serialize, Deserialize)]
pub struct PlayerDespawnEvent {
    #[entity]
    #[missing = "drop"]
    pub player_entity: Entity,
    pub player_id: ClientId,
}

/// The player that received this is ready to spawn into the world.
#[client_bound]
#[derive(Serialize, Deserialize)]
pub struct PlayerReadyEvent {
    pub own_client_it: ClientId,
    pub players_online_count: i16,
}
