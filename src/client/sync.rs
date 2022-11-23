use bevy::{
    prelude::{
        default, BuildChildren, Commands, DespawnRecursiveExt, Entity, EventReader,
        ParallelSystemDescriptorCoercion, PbrBundle, Plugin, Query, Res, ResMut, Transform,
    },
    transform::TransformBundle,
};
use bevy_rapier3d::prelude::Velocity;
use spacegame_core::network_id::NetworkIdMap;

use crate::{
    events::ship::{BlockRemoveEvent, BlockUpdateEvent, LoadShipEvent},
    model::{
        block::{BlockBundle, BlockType},
        block_map::{BlockMap, BlockPosition, BlockRotation},
        ship::{ShipBundle, ShipName},
    },
    resources::block_registry::BlockRegistry,
    shared::events::{
        generic::GenericPositionSyncEvent,
        ship::{SyncShipBlocksEvent, SyncShipEvent, SyncShipPositionEvent},
    },
};

use super::labels::UpdateLabels;

pub struct SyncPlugin;

impl Plugin for SyncPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_system(on_sync_ship.label(UpdateLabels::Sync))
            .add_system(on_sync_ship_position.label(UpdateLabels::Sync))
            .add_system(on_load_ship)
            .add_system(on_generic_position_sync.label(UpdateLabels::Sync))
            .add_system(on_block_update)
            .add_system(on_block_remove);
    }

    fn name(&self) -> &str {
        "sync_plugin"
    }
}

fn on_load_ship(
    mut commands: Commands,
    block_registry: Res<BlockRegistry>,
    mut events: EventReader<LoadShipEvent>,
) {
    for event in events.iter() {
        commands
            .entity(event.ship_entity)
            .insert_bundle(ShipBundle {
                block_map: event.block_map.clone(),
                transform_bundle: TransformBundle {
                    local: event.transform,
                    ..default()
                },
                velocity: event.velocity,
                ship_name: ShipName {
                    name: event.name.clone(),
                },
                ..default()
            });
        sync_blocks(
            &mut commands,
            &block_registry,
            &BlockMap::new(),
            &event.block_map,
            &event.ship_entity,
        );
    }
}

fn on_sync_ship(
    mut commands: Commands,
    block_registry: Res<BlockRegistry>,
    mut network_ids: ResMut<NetworkIdMap>,
    mut ship_events: EventReader<SyncShipEvent>,
    mut ship_query: Query<(&mut Transform, &mut Velocity, &mut BlockMap)>,
) {
    for event in ship_events.iter() {
        if let Ok((mut transform, mut velocity, mut block_map)) =
            ship_query.get_mut(event.ship_entity)
        {
            *transform = event.transform;
            *velocity = event.velocity;
            *block_map = event.block_map.clone();
        }
        // // Spawn new ship
        // let ship_entity = commands
        //     .spawn_bundle(ShipBundle {
        //         transform_bundle: TransformBundle {
        //             local: event.transform,
        //             ..default()
        //         },
        //         velocity: event.velocity,
        //         ..default()
        //     })
        //     .insert(event.ship_network_id)
        //     .id();

        // network_ids.insert_with_network_id(ship_entity, event.ship_network_id);

        // let map = sync_blocks(
        //     &mut commands,
        //     &block_registry,
        //     &BlockMap::new(),
        //     &event.block_map,
        //     &ship_entity,
        // );

        // commands.entity(ship_entity).insert(map);
    }
}

fn on_sync_ship_position(
    mut network_ids: ResMut<NetworkIdMap>,
    mut ship_events: EventReader<SyncShipPositionEvent>,
    mut ship_query: Query<(&mut Transform, &mut Velocity)>,
) {
    for event in ship_events.iter() {
        if let Ok((mut transform, mut velocity)) = ship_query.get_mut(event.ship_entity) {
            *transform = event.transform;
            *velocity = event.velocity;
        }
    }
}

fn on_sync_ship_blocks(
    mut network_ids: ResMut<NetworkIdMap>,
    mut ship_events: EventReader<SyncShipBlocksEvent>,
    mut ship_query: Query<(Entity, &mut BlockMap)>,
) {
}

fn on_block_update(
    mut commands: Commands,
    block_registry: Res<BlockRegistry>,
    mut network_ids: ResMut<NetworkIdMap>,
    mut events: EventReader<BlockUpdateEvent>,
    mut ship_query: Query<&mut BlockMap>,
) {
    for event in events.iter() {
        let mut block_map = ship_query.get_mut(event.ship_entity).unwrap();
        spawn_block(
            &mut commands,
            &block_registry,
            &mut block_map,
            &event.ship_entity,
            event.block_position,
            event.block_type,
            event.block_rotation,
        );
    }
}

fn on_block_remove(
    mut commands: Commands,
    block_registry: Res<BlockRegistry>,
    mut network_ids: ResMut<NetworkIdMap>,
    mut events: EventReader<BlockRemoveEvent>,
    mut ship_query: Query<&mut BlockMap>,
) {
    for event in events.iter() {
        let mut block_map = ship_query.get_mut(event.ship_entity).unwrap();
        if let Some(entity) = block_map.remove(&event.block_position) {
            commands.entity(entity).despawn_recursive();
        }
    }
}

fn sync_blocks(
    commands: &mut Commands,
    block_registry: &BlockRegistry,
    old_block_map: &BlockMap,
    new_block_map: &BlockMap,
    ship_entity: &Entity,
) -> BlockMap {
    for (pos, entry) in old_block_map.entries() {
        commands.entity(entry.entity).despawn_recursive();
        commands
            .entity(*ship_entity)
            .remove_children(&[entry.entity]);
    }

    let mut return_map = BlockMap::new();
    for (pos, entry) in new_block_map.entries() {
        spawn_block(
            commands,
            block_registry,
            &mut return_map,
            ship_entity,
            *pos,
            entry.block_type,
            entry.block_rotation,
        );
    }

    return_map
}

fn spawn_block(
    commands: &mut Commands,
    block_registry: &BlockRegistry,
    block_map: &mut BlockMap,
    ship_entity: &Entity,
    block_position: BlockPosition,
    block_type: BlockType,
    block_rotation: BlockRotation,
) -> Entity {
    let block_entity = create_block(commands, block_registry, block_position, block_type);
    commands.entity(*ship_entity).add_child(block_entity);
    if let Some(old_block) = block_map.set(block_entity, block_type, block_position, block_rotation)
    {
        commands.entity(old_block.entity).despawn_recursive();
    }

    block_entity
}

fn create_block(
    commands: &mut Commands,
    block_registry: &BlockRegistry,
    block_position: BlockPosition,
    block_type: BlockType,
) -> Entity {
    commands
        .spawn_bundle(BlockBundle {
            block_type: block_type,
            block_position: block_position,
            pbr_bundle: PbrBundle {
                transform: block_position.into(),
                material: block_registry.get_material(block_type),
                mesh: block_registry.get_mesh(block_type),
                ..default()
            },
            ..default()
        })
        .id()
}

fn on_generic_position_sync(
    network_ids: Res<NetworkIdMap>,
    mut query: Query<(&mut Transform, &mut Velocity)>,
    mut events: EventReader<GenericPositionSyncEvent>,
) {
    for event in events.iter() {
        if let Ok((mut transform, mut velocity)) = query.get_mut(event.entity) {
            *transform = event.transform;
            *velocity = event.velocity;
        }
    }
}
