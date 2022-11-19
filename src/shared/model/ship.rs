use bevy::prelude::{Bundle, Component, ComputedVisibility, Entity, Visibility};
use bevy::transform::TransformBundle;
use bevy_rapier3d::dynamics::Velocity;
use bevy_rapier3d::prelude::{Ccd, Damping, ExternalForce, ExternalImpulse, RigidBody, Sleeping};
use serde::{Deserialize, Serialize};
use spaccegame_proc_macros::NetworkEvent;

use crate::model::block_map::BlockMap;

#[derive(Component)]
pub struct Thrust {
    pub t: f32,
}

impl Default for Thrust {
    fn default() -> Self {
        Self { t: 1. }
    }
}

#[derive(Component)]
pub struct Gimbal {
    pub t: f32,
}

impl Default for Gimbal {
    fn default() -> Self {
        Self { t: 0.25 }
    }
}

#[derive(Component)]
pub struct ShipName {
    pub name: String,
}

impl Default for ShipName {
    fn default() -> Self {
        Self {
            name: "Test Name".into(),
        }
    }
}

#[derive(Component)]
pub struct Ship;

#[derive(Bundle)]
pub struct ShipBundle {
    pub ship: Ship,
    pub block_map: BlockMap,
    pub rigid_body: RigidBody,
    #[bundle]
    pub transform_bundle: TransformBundle,
    pub velocity: Velocity,
    pub visibility: Visibility,
    pub computed_visibility: ComputedVisibility,
    pub sleeping: Sleeping,
    pub ccd: Ccd,
    pub ship_name: ShipName,
    pub thrust: Thrust,
    pub gimbal: Gimbal,
    pub damping: Damping,
    pub impulse: ExternalImpulse,
    pub force: ExternalForce,
}

impl Default for ShipBundle {
    fn default() -> Self {
        Self {
            ship: Ship,
            block_map: BlockMap::new(),
            rigid_body: RigidBody::Dynamic,
            transform_bundle: TransformBundle::default(),
            velocity: Velocity::zero(),
            visibility: Visibility::visible(),
            computed_visibility: ComputedVisibility::default(),
            sleeping: Sleeping::disabled(),
            ccd: Ccd::enabled(),
            ship_name: ShipName::default(),
            thrust: Thrust { t: 1000. },
            gimbal: Gimbal { t: 10. },
            damping: Damping {
                linear_damping: 1.,
                angular_damping: 1.,
            },
            impulse: ExternalImpulse::default(),
            force: ExternalForce::default(),
        }
    }
}

#[derive(Component, Serialize, Deserialize, Debug, NetworkEvent)]
pub struct Pilot {
    #[networkEntity]
    #[dropIfNone]
    pub player_entity: Entity,
    pub player_id: u64,
}
