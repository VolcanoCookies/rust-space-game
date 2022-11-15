use bevy::prelude::{Component, Name, Transform};
use serde::{Deserialize, Serialize};

use crate::shared::networking::network_id::NetworkId;

use crate::shared::remote_refs::TransformDef;

#[derive(Debug, Serialize, Deserialize)]
pub struct PlayerMoveEvent {
    pub player_network_id: NetworkId,
    #[serde(with = "TransformDef")]
    pub transform: Transform,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PlayerSpawnEvent {
    pub player_network_id: NetworkId,
    pub player_name: String,
    #[serde(with = "TransformDef")]
    pub transform: Transform,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PlayerDespawnEvent {
    pub player_network_id: NetworkId,
}
