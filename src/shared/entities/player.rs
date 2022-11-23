use bevy::{
    prelude::{default, Bundle, Component, Mesh, Name},
    render::mesh,
};
use bevy_rapier3d::prelude::{Collider, ComputedColliderShape, Damping, Dominance};
use spacegame_core::message::ClientId;

use super::physic_object::PhysicsObjectBundle;

#[derive(Bundle)]
pub struct PlayerBundle {
    #[bundle]
    pub physics_object: PhysicsObjectBundle,
    pub name: Name,
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
            marker: PlayerMarker,
        }
    }
}

// Marker for any player entity, not just our own
#[derive(Component)]
pub struct PlayerMarker;

#[derive(Component)]
pub struct PlayerClientId(pub ClientId);
