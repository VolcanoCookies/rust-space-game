use bevy::prelude::{Commands, DespawnRecursiveExt, EventReader, Plugin, Query, Res, ResMut, With};
use bevy_renet::renet::RenetServer;

use crate::{
    client::controller::ShipMarker,
    events::ship::{
        BlockRemoveEvent, BlockUpdateEvent, EnterShipEvent, PlaceBlockEvent, RemoveBlockEvent,
    },
    model::{
        block::BlockBundle,
        block_map::{self, BlockMap},
        ship::{Pilot, Ship},
    },
    networking::{message::ServerMessage, network_id::NetworkIdMap},
};

use super::networking::NetworkServer;

pub struct ShipPlugin;

impl Plugin for ShipPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_system(on_block_place).add_system(on_block_remove);
    }

    fn name(&self) -> &str {
        "ship_plugin"
    }
}

fn on_block_place(
    mut commands: Commands,
    network_ids: Res<NetworkIdMap>,
    mut server: ResMut<RenetServer>,
    mut events: EventReader<PlaceBlockEvent>,
    mut query: Query<&mut BlockMap>,
) {
    for event in events.iter() {
        if let Some(ship_entity) = network_ids.from_network(event.ship_network_id) {
            let mut block_map = query.get_mut(ship_entity).unwrap();
            // TODO: Check if an identical block already exists here
            let block_entity = commands
                .spawn_bundle(BlockBundle::new(
                    event.block_type,
                    event.block_position,
                    event.block_rotation,
                ))
                .id();

            NetworkServer::broadcast_message(
                &mut server,
                ServerMessage::BlockUpdate(BlockUpdateEvent {
                    ship_network_id: event.ship_network_id,
                    block_type: event.block_type,
                    block_position: event.block_position,
                    block_rotation: event.block_rotation,
                }),
            );

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
}

fn on_block_remove(
    mut commands: Commands,
    network_ids: Res<NetworkIdMap>,
    mut server: ResMut<RenetServer>,
    mut events: EventReader<RemoveBlockEvent>,
    mut query: Query<&mut BlockMap>,
) {
    for event in events.iter() {
        if let Some(ship_entity) = network_ids.from_network(event.ship_network_id) {
            let (mut block_map) = query.get_mut(ship_entity).unwrap();

            if let Some(old_block_entity) = block_map.remove(&event.block_position) {
                NetworkServer::broadcast_message(
                    &mut server,
                    ServerMessage::BlockRemove(BlockRemoveEvent {
                        ship_network_id: event.ship_network_id,
                        block_position: event.block_position,
                    }),
                );

                commands.entity(old_block_entity).despawn_recursive();
            }
        }
    }
}

fn on_enter_ship(
    mut commands: Commands,
    network_ids: Res<NetworkIdMap>,
    mut server: ResMut<RenetServer>,
    mut events: EventReader<EnterShipEvent>,
    pilot_query: Query<Option<&Pilot>, With<Ship>>,
) {
    // for event in events.iter() {
    //     if let Some(ship_entity) = network_ids.from_network(event.ship_network_id) {
    //         let (opt_pilot) = pilot_query.get(ship_entity).unwrap();
    // 		if opt_pilot.is_none() {
    // 			NetworkServer::send_message(&mut server, event, message)
    // 		} else {
    // 			// Send back error packet
    // 		}
    //     }
    // }
}
