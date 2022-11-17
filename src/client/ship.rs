use bevy::prelude::{
    default, warn, Commands, Entity, EventReader, PbrBundle, Query, ResMut, Transform,
};
use bevy_rapier3d::prelude::Velocity;

use crate::{
    model::{
        block::{BlockBundle, BlockType},
        block_map::BlockPosition,
    },
    resources::block_registry::BlockRegistry,
    shared::{events::ship::SyncShipPositionEvent, networking::network_id::NetworkIdMap},
};

pub fn sync_ship_position(
    mut commands: Commands,
    mut network_ids: ResMut<NetworkIdMap>,
    mut events: EventReader<SyncShipPositionEvent>,
    mut ship_query: Query<(&mut Transform, &mut Velocity)>,
) {
    for event in events.iter() {
        if let Some(ship_entity) = network_ids.from_network(event.ship_network_id) {
            // Sync existing ship
            if let Ok((mut ship_transform, mut ship_velocity)) = ship_query.get_mut(ship_entity) {
                *ship_transform = event.transform;
                *ship_velocity = event.velocity;
            }
        } else {
            // Spawn new ship
            warn!("Got sync ship event for unknown ship");
        }
    }
}

fn spawn_block(
    commands: &mut Commands,
    block_registry: &BlockRegistry,
    block_position: BlockPosition,
    block_type: BlockType,
) -> Entity {
    commands
        .spawn_bundle(BlockBundle {
            block_type,
            block_position,
            pbr_bundle: PbrBundle {
                transform: block_position.into(),
                mesh: block_registry.get_mesh(block_type),
                material: block_registry.get_material(block_type),
                ..default()
            },
            ..default()
        })
        .id()
}
