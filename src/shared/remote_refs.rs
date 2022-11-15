use bevy::prelude::{Name, Quat, Transform, Vec3};
use bevy_rapier3d::{dynamics::Velocity, prelude::Vect};
use serde::{Deserialize, Serialize};

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
