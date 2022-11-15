use bevy::prelude::Transform;
use bevy_rapier3d::prelude::Velocity;
use serde::{Deserialize, Serialize};

use crate::shared::networking::network_id::NetworkId;

use crate::shared::remote_refs::{TransformDef, VelocityDef};

#[derive(Debug, Serialize, Deserialize)]
pub struct GenericPositionSyncEvent {
    pub network_id: NetworkId,
    #[serde(with = "TransformDef")]
    pub transform: Transform,
    #[serde(with = "VelocityDef")]
    pub velocity: Velocity,
}
