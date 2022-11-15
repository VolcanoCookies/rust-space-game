use bevy::prelude::{Changed, Entity, EventReader, Or, Plugin, Query, ResMut, Transform};
use bevy_rapier3d::prelude::Velocity;
use bevy_renet::renet::{RenetServer, ServerEvent};

use crate::{
    model::block_map::BlockMap,
    shared::{
        events::{
            generic::GenericPositionSyncEvent,
            ship::{SyncShipEvent, SyncShipPositionEvent},
        },
        networking::{
            message::{NetworkMessage, ServerMessage},
            network_id::{self, NetworkId, NetworkIdMap},
        },
    },
};

pub struct SyncPlugin;

impl Plugin for SyncPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_system(on_client_connect);
    }

    fn name(&self) -> &str {
        "sync_plugin"
    }
}

fn on_client_connect(
    mut network_ids: ResMut<NetworkIdMap>,
    mut server: ResMut<RenetServer>,
    mut server_events: EventReader<ServerEvent>,
    ship_query: Query<(Entity, &Transform, &Velocity, &BlockMap)>,
) {
    for event in server_events.iter() {
        match event {
            ServerEvent::ClientConnected(client_id, _) => {
                for (entity, transform, velocity, block_map) in ship_query.iter() {
                    let network_id = network_ids.get(entity);
                    let message = ServerMessage::SyncShip(SyncShipEvent {
                        ship_network_id: network_id,
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

fn generic_position_sync(
    mut server: ResMut<RenetServer>,
    query: Query<(&NetworkId, &Transform, &Velocity), Or<(Changed<Transform>, Changed<Velocity>)>>,
) {
    for (network_id, transform, velocity) in query.iter() {
        let message = ServerMessage::GenericPositionSync(GenericPositionSyncEvent {
            network_id: *network_id,
            transform: *transform,
            velocity: *velocity,
        });
        server.broadcast_message(message.channel_id(), bincode::serialize(&message).unwrap());
    }
}
