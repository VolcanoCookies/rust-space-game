use bevy::prelude::{Changed, Component, EventReader, Or, Plugin, Query, ResMut, Transform, With};
use bevy_rapier3d::prelude::Velocity;
use bevy_renet::renet::{RenetServer, ServerEvent};
use spacegame_core::network_id::NetworkId;

use crate::{
    model::block_map::BlockMap,
    shared::{
        events::{generic::GenericPositionSyncEvent, ship::SyncShipEvent},
        networking::message::{NetworkMessage, ServerMessage},
    },
};

pub struct SyncPlugin;

impl Plugin for SyncPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_system(on_client_connect)
            .add_system(generic_position_sync);
    }

    fn name(&self) -> &str {
        "sync_plugin"
    }
}

fn on_client_connect(
    mut server: ResMut<RenetServer>,
    mut server_events: EventReader<ServerEvent>,
    ship_query: Query<(&NetworkId, &Transform, &Velocity, &BlockMap)>,
) {
    for event in server_events.iter() {
        match event {
            ServerEvent::ClientConnected(client_id, _) => {
                for (network_id, transform, velocity, block_map) in ship_query.iter() {
                    let message = ServerMessage::SyncShip(SyncShipEvent {
                        ship_entity: network_id.into(),
                        transform: transform.clone(),
                        velocity: velocity.clone(),
                        block_map: block_map.clone(),
                    });
                    send(&mut server, client_id, &message);
                }
            }
            ServerEvent::ClientDisconnected(_) => {}
        }
    }
}

fn send<T: ?Sized>(server: &mut RenetServer, client_id: &u64, message: &T)
where
    T: serde::Serialize + NetworkMessage,
{
    server.send_message(
        *client_id,
        message.channel_id(),
        bincode::serialize(message).unwrap(),
    )
}

#[derive(Component)]
pub struct GenericPositionSyncMarker;

fn generic_position_sync(
    mut server: ResMut<RenetServer>,
    query: Query<
        (&NetworkId, &Transform, &Velocity),
        (
            With<GenericPositionSyncMarker>,
            Or<(Changed<Transform>, Changed<Velocity>)>,
        ),
    >,
) {
    for (network_id, transform, velocity) in query.iter() {
        let message = ServerMessage::GenericPositionSync(GenericPositionSyncEvent {
            entity: network_id.into(),
            transform: *transform,
            velocity: *velocity,
        });
        server.broadcast_message(message.channel_id(), bincode::serialize(&message).unwrap());
    }
}
