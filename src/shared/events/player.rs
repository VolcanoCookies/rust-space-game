use bevy::prelude::{Component, Entity, Name, Transform};
use serde::{Deserialize, Serialize};
use spacegame_proc_macros::client_bound;

use crate::shared::remote_refs::TransformDef;

#[derive(Debug, Serialize, Deserialize)]
pub struct PlayerMoveEvent {
    pub client_id: u64,
    #[serde(with = "TransformDef")]
    pub transform: Transform,
}

#[derive(Debug, Serialize, Deserialize)]
#[client_bound]
pub struct PlayerSpawnEvent {
    #[entity]
    #[missing = "create"]
    pub player_entity: Entity,
    pub player_name: String,
    #[serde(with = "TransformDef")]
    pub transform: Transform,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PlayerDespawnEvent {
    pub player_entity: Entity,
}
