use bevy::prelude::{
    trace, BuildChildren, Changed, Commands, DespawnRecursiveExt, Entity, EventReader,
    ParallelSystemDescriptorCoercion, Plugin, Query, ResMut, Transform, With,
};
use bevy_rapier3d::prelude::{ExternalForce, Velocity};
use spacegame_core::message::ServerMessageOutQueue;

use crate::{
    events::ship::{
        BlockRemoveEvent, BlockUpdateEvent, EnteredShipEvent, LeftShipEvent, ShipMoveEvent,
        SyncShipPositionEvent, TryEnterShipEvent, TryLeaveShipEvent, UnloadShipEvent,
    },
    model::{
        block::BlockBundle,
        block_map::BlockMap,
        ship::{Pilot, Ship},
    },
};

use super::labels::UpdateLabels;

pub struct ShipPlugin;

impl Plugin for ShipPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_system(on_block_update)
            .add_system(on_block_remove)
            .add_system(on_ship_move)
            .add_system(on_try_enter_ship)
            .add_system(on_try_leave_ship)
            .add_system(sync_ship_position_velocity.label(UpdateLabels::Sync));
    }

    fn name(&self) -> &str {
        "ship_plugin"
    }
}

fn on_block_update(
    mut commands: Commands,
    mut events: EventReader<BlockUpdateEvent>,
    mut query: Query<&mut BlockMap>,
    mut block_update_queue: ResMut<ServerMessageOutQueue<BlockUpdateEvent>>,
) {
    for event in events.iter() {
        let mut block_map = query.get_mut(event.ship_entity).unwrap();
        // TODO: Check if an identical block already exists here
        let block_entity = commands
            .spawn_bundle(BlockBundle::new(
                event.block_type,
                event.block_position,
                event.block_rotation,
            ))
            .id();

        commands.entity(event.ship_entity).add_child(block_entity);

        block_update_queue.broadcast(BlockUpdateEvent {
            ship_entity: event.ship_entity,
            block_type: event.block_type,
            block_position: event.block_position,
            block_rotation: event.block_rotation,
            client_id: 0,
        });

        if let Some(old_block) = block_map.set(
            block_entity,
            event.block_type,
            event.block_position,
            event.block_rotation,
        ) {
            commands.entity(old_block.entity).despawn_recursive();
        }
    }
}

fn on_block_remove(
    mut commands: Commands,
    mut events: EventReader<BlockRemoveEvent>,
    mut query: Query<(Entity, &mut BlockMap)>,
    mut block_remove_queue: ResMut<ServerMessageOutQueue<BlockRemoveEvent>>,
) {
    for event in events.iter() {
        let (ship_entity, mut block_map) = query.get_mut(event.ship_entity).unwrap();

        if let Some(old_block_entity) = block_map.remove(&event.block_position) {
            block_remove_queue.broadcast(BlockRemoveEvent {
                ship_entity,
                block_position: event.block_position,
                client_id: 0,
            });

            commands.entity(old_block_entity).despawn_recursive();
        }
    }
}

fn on_try_enter_ship(
    mut events: EventReader<TryEnterShipEvent>,
    mut queue: ResMut<ServerMessageOutQueue<EnteredShipEvent>>,
    mut pilot_query: Query<&mut Pilot, With<Ship>>,
) {
    for event in events.iter() {
        let mut pilot = pilot_query.get_mut(event.ship_entity).unwrap();
        trace!(
            "{:?} tried to enter ship {:?}",
            event.client_id,
            event.ship_entity
        );
        match *pilot {
            Pilot::None => {
                // TODO: Check that player can in-fact enter this ship, distance, faction, etc.
                *pilot = Pilot::Pilot(event.client_id);
                queue.broadcast(EnteredShipEvent {
                    ship_entity: event.ship_entity,
                    player_id: event.client_id,
                });
                trace!("{:?} entered ship {:?}", event.client_id, event.ship_entity);
            }
            _ => {}
        }
    }
}

fn on_try_leave_ship(
    mut events: EventReader<TryLeaveShipEvent>,
    mut queue: ResMut<ServerMessageOutQueue<LeftShipEvent>>,
    mut pilot_query: Query<&mut Pilot, With<Ship>>,
) {
    for event in events.iter() {
        let mut pilot = pilot_query.get_mut(event.ship_entity).unwrap();
        match *pilot {
            Pilot::Pilot(client_id) => {
                trace!(
                    "{:?} tried to exit ship {:?}",
                    event.client_id,
                    event.ship_entity
                );
                if client_id == event.client_id {
                    *pilot = Pilot::None;
                    queue.broadcast(LeftShipEvent {
                        ship_entity: event.ship_entity,
                        player_id: event.client_id,
                    });
                    trace!("{:?} exited ship {:?}", event.client_id, event.ship_entity,);
                }
            }
            Pilot::None => {}
        }
    }
}

fn on_ship_move(
    mut events: EventReader<ShipMoveEvent>,
    mut query: Query<(&Pilot, &mut ExternalForce), With<Ship>>,
) {
    for event in events.iter() {
        if let Ok((Pilot::Pilot(pilot_id), mut force)) = query.get_mut(event.ship_entity) {
            if *pilot_id == event.client_id {
                *force = event.force;
            }
        }
    }
}

fn sync_ship_position_velocity(
    query: Query<
        (Entity, &Transform, &Velocity),
        (Changed<Transform>, Changed<Velocity>, With<Ship>),
    >,
    mut queue: ResMut<ServerMessageOutQueue<SyncShipPositionEvent>>,
) {
    for (ship_entity, transform, velocity) in query.iter() {
        queue.broadcast(SyncShipPositionEvent {
            ship_entity,
            transform: *transform,
            velocity: *velocity,
        });
    }
}

fn despawn_empty_ship(
    mut commands: Commands,
    query: Query<(Entity, &BlockMap, Option<&Pilot>), With<Ship>>,
    mut unload_ship_queue: ResMut<ServerMessageOutQueue<UnloadShipEvent>>,
) {
    for (ship_entity, block_map, pilot) in query.iter() {
        if block_map.block_count == 0 {
            unload_ship_queue.broadcast(UnloadShipEvent { ship_entity });
            commands.entity(ship_entity).despawn_recursive();
        }
    }
}
