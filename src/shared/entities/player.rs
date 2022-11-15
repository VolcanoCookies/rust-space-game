use bevy::{
    ecs::bundle,
    prelude::{default, Bundle, Component, Mesh, Name},
    render::mesh,
    transform::TransformBundle,
};
use bevy_rapier3d::prelude::{
    Ccd, Collider, ComputedColliderShape, Damping, Dominance, ExternalForce, ExternalImpulse,
    RigidBody, Sleeping, Velocity,
};

use crate::shared::networking::network_id::NetworkId;

use super::physic_object::PhysicsObjectBundle;

#[derive(Bundle)]
pub struct PlayerBundle {
    #[bundle]
    pub physics_object: PhysicsObjectBundle,
    pub name: Name,
    pub network_id: NetworkId,
    pub dominance: Dominance,
    pub marker: PlayerMarker,
}

impl Default for PlayerBundle {
    fn default() -> Self {
        Self {
            physics_object: PhysicsObjectBundle {
                damping: Damping {
                    linear_damping: 2.,
                    angular_damping: 2.,
                },
                collider: Collider::from_bevy_mesh(
                    &Mesh::from(mesh::shape::Capsule::default()),
                    &ComputedColliderShape::TriMesh,
                )
                .unwrap(),
                ..default()
            },
            dominance: Dominance { groups: -1 },
            name: Default::default(),
            network_id: Default::default(),
            marker: PlayerMarker,
        }
    }
}

#[derive(Component)]
pub struct PlayerMarker;
