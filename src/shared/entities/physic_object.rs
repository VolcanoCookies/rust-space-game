use bevy::{prelude::Bundle, transform::TransformBundle};
use bevy_rapier3d::prelude::{
    Ccd, Collider, Damping, ExternalForce, ExternalImpulse, RigidBody, Sleeping, Velocity,
};

#[derive(Bundle, Default)]
pub struct PhysicsObjectBundle {
    #[bundle]
    pub transform_bundle: TransformBundle,
    pub velocity: Velocity,
    pub damping: Damping,
    pub impulse: ExternalImpulse,
    pub force: ExternalForce,
    pub collider: Collider,
    pub rigid_body: RigidBody,
    pub sleeping: Sleeping,
    pub ccd: Ccd,
}
