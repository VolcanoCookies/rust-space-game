use bevy::{
    prelude::{Component, Quat, Vec3},
    reflect::{FromReflect, Reflect},
};
use serde::{Deserialize, Serialize};

#[derive(
    Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Reflect, FromReflect, Component,
)]
pub struct Velocity {
    pub linvel: Vec3,
    pub angvel: Vec3,
}

#[derive(
    Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Reflect, FromReflect, Component,
)]
pub struct Force {
    pub force: Vec3,
    pub torque: Vec3,
}

#[derive(
    Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Reflect, FromReflect, Component,
)]
pub struct Impulse {
    pub impulse: Vec3,
    pub torque_impulse: Vec3,
}

#[derive(
    Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Reflect, FromReflect, Component,
)]
pub struct Dominance {
    pub group: i8,
}

#[derive(
    Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Reflect, FromReflect, Component,
)]
pub struct Sleeping {
    /// Linear velocity bellow which a body is allowed to fall asleep
    pub linear_threshold: Vec3,
    /// Angular velocity bellow which a body is allowed to fall asleep
    pub angular_threshold: Vec3,
    /// If the body is sleeping
    pub sleeping: bool,
}

#[derive(
    Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Reflect, FromReflect, Component,
)]
pub struct Damping {
    pub linear_damping: Vec3,
    pub angular_damping: Vec3,
}

#[derive(
    Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Reflect, FromReflect, Component,
)]
pub struct MassProperties {
    pub center_of_map: Vec3,
    pub mass: f32,
    pub principal_intertial_rot: Quat,
    pub principal_inertial: Vec3,
}

#[derive(
    Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Reflect, FromReflect, Component,
)]
pub struct RigidBody;
