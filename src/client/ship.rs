use bevy::prelude::{
    default, warn, BuildChildren, Commands, DespawnRecursiveExt, Entity, EventReader, PbrBundle,
    Query, Res, ResMut, Transform,
};
use bevy_rapier3d::prelude::Velocity;

use crate::{
    model::{
        block::{self, BlockBundle, BlockType},
        block_map::{BlockMap, BlockPosition},
        ship::Ship,
    },
    resources::block_registry::BlockRegistry,
    shared::{
        events::ship::{AddBlockEvent, RemoveBlockEvent, SyncShipPositionEvent},
        networking::network_id::NetworkIdMap,
    },
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

pub fn add_block(
    mut commands: Commands,
    block_registry: Res<BlockRegistry>,
    network_ids: Res<NetworkIdMap>,
    mut events: EventReader<AddBlockEvent>,
    mut block_map_query: Query<&mut BlockMap>,
) {
    for event in events.iter() {
        if let Some(ship_entity) = network_ids.from_network(event.ship_network_id) {
            if let Ok((mut block_map)) = block_map_query.get_mut(ship_entity) {
                let block_entity = spawn_block(
                    &mut commands,
                    &block_registry,
                    event.block_position,
                    event.block_type,
                );

                commands.entity(ship_entity).add_child(block_entity);
                block_map.set(event.block_position, event.block_type, block_entity);
            }
        } else {
            warn!("Got add block event for unknown ship");
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

fn remove_block(
    mut commands: Commands,
    network_ids: Res<NetworkIdMap>,
    mut events: EventReader<RemoveBlockEvent>,
    mut block_map_query: Query<&mut BlockMap>,
) {
    for event in events.iter() {
        if let Some(ship_entity) = network_ids.from_network(event.ship_network_id) {
            if let Ok(mut block_map) = block_map_query.get_mut(ship_entity) {
                if let Some(block_entity) = block_map.remove(&event.block_position) {
                    commands
                        .entity(ship_entity)
                        .remove_children(&[block_entity]);
                    commands.entity(block_entity).despawn_recursive();
                }
            }
        } else {
            warn!("Got remove block event for unknown ship");
        }
    }
}
