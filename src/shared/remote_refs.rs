use bevy::prelude::{Quat, Transform, Vec3};
use bevy_rapier3d::{dynamics::Velocity, prelude::Vect};
use serde::{Deserialize, Serialize};

use bevy_rapier3d::dynamics::ExternalForce;

#[derive(Serialize, Deserialize)]
#[serde(remote = "Transform")]
pub struct TransformDef {
    pub translation: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

#[derive(Serialize, Deserialize)]
#[serde(remote = "Velocity")]
pub struct VelocityDef {
    pub linvel: Vect,
    pub angvel: Vect,
}

#[derive(Serialize, Deserialize)]
#[serde(remote = "ExternalForce")]
pub struct ExternalForceDef {
    pub force: Vect,
    pub torque: Vect,
}
