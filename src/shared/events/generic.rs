use bevy::prelude::{Entity, Transform};
use bevy_rapier3d::prelude::Velocity;
use serde::{Deserialize, Serialize};
use spacegame_proc_macros::{client_bound, NetworkEvent};

use crate::shared::remote_refs::{TransformDef, VelocityDef};

#[derive(Debug, Serialize, Deserialize)]
#[client_bound]
pub struct GenericPositionSyncEvent {
    #[entity]
    #[missing = "drop"]
    pub entity: Entity,
    #[serde(with = "TransformDef")]
    pub transform: Transform,
    #[serde(with = "VelocityDef")]
    pub velocity: Velocity,
}
