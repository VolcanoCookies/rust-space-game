use bevy::prelude::{Entity, Transform};
use bevy_rapier3d::prelude::Velocity;
use serde::{Deserialize, Serialize};
use spacegame_proc_macros::client_bound;

use crate::shared::remote_refs::{TransformDef, VelocityDef};

#[client_bound]
#[derive(Serialize, Deserialize)]
pub struct GenericPositionSyncEvent {
    #[entity]
    #[missing = "drop"]
    pub entity: Entity,
    #[serde(with = "TransformDef")]
    pub transform: Transform,
    #[serde(with = "VelocityDef")]
    pub velocity: Velocity,
}

pub struct UnbindPositionEvent {
    pub parent: Entity,
    pub child: Entity,
}

pub struct BindPositionEvent {
    pub parent: Entity,
    pub child: Entity,
}
