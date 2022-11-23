use bevy::prelude::{
    BuildChildren, Changed, Commands, Component, Entity, EventReader, GlobalTransform, Or, Plugin,
    Query, ResMut, Transform, With,
};
use bevy_rapier3d::prelude::Velocity;
use bevy_renet::renet::ServerEvent;
use spacegame_core::{message::ServerMessageOutQueue, network_id::NetworkId};

use crate::{
    events::{
        generic::{BindPositionEvent, UnbindPositionEvent},
        ship::LoadShipEvent,
    },
    model::block_map::BlockMap,
    shared::events::generic::GenericPositionSyncEvent,
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
    mut server_events: EventReader<ServerEvent>,
    ship_query: Query<(Entity, &Transform, &Velocity, &BlockMap)>,
    mut ship_queue: ResMut<ServerMessageOutQueue<LoadShipEvent>>,
) {
    for event in server_events.iter() {
        match event {
            ServerEvent::ClientConnected(client_id, _) => {
                for (ship_entity, transform, velocity, block_map) in ship_query.iter() {
                    ship_queue.send(
                        client_id,
                        LoadShipEvent {
                            ship_entity,
                            transform: transform.clone(),
                            velocity: velocity.clone(),
                            block_map: block_map.clone(),
                            name: String::from("some ship generic ass name"),
                        },
                    );
                }
            }
            ServerEvent::ClientDisconnected(_) => {}
        }
    }
}

#[derive(Component)]
pub struct GenericPositionSyncMarker;

fn generic_position_sync(
    query: Query<
        (&NetworkId, &Transform, &Velocity),
        (
            With<GenericPositionSyncMarker>,
            Or<(Changed<Transform>, Changed<Velocity>)>,
        ),
    >,
    mut generic_sync_position_queue: ResMut<ServerMessageOutQueue<GenericPositionSyncEvent>>,
) {
    for (network_id, transform, velocity) in query.iter() {
        generic_sync_position_queue.broadcast(GenericPositionSyncEvent {
            entity: network_id.into(),
            transform: *transform,
            velocity: *velocity,
        });
    }
}

