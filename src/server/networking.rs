use std::{
    net::{SocketAddr, UdpSocket},
    time::SystemTime,
};

use bevy::{
    prelude::{
        default, Commands, DespawnRecursiveExt, Entity, EventReader, Name, Plugin, Query, ResMut,
        Transform, With,
    },
    transform::TransformBundle,
};

use bevy_renet::renet::{
    RenetConnectionConfig, RenetServer, ServerAuthentication, ServerConfig, ServerEvent,
};
use local_ip_address::local_ip;
use spacegame_core::{
    message::ServerMessageOutQueue,
    network_id::NetworkIdMap,
    server::{AppServerNetworkTrait, ServerNetworkPlugin},
};

use crate::{
    entities::{physic_object::PhysicsObjectBundle, player::PlayerClientId},
    events::{
        player::{PlayerDespawnEvent, PlayerMoveEvent, PlayerReadyEvent},
        ship::{
            BlockRemoveEvent, BlockUpdateEvent, EnteredShipEvent, LeftShipEvent, LoadShipEvent,
            ShipMoveEvent, SyncShipBlocksEvent, SyncShipEvent, SyncShipPositionEvent,
            TryEnterShipEvent, TryLeaveShipEvent, UnloadShipEvent,
        },
    },
    shared::{
        entities::player::{PlayerBundle, PlayerMarker},
        events::{generic::GenericPositionSyncEvent, player::PlayerSpawnEvent},
        networking::{player_id::PlayerIdMap, plugin::NetworkingPlugin},
    },
    PROTOCOL_ID,
};

pub struct ServerNetworkingPlugin;

impl Plugin for ServerNetworkingPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugin(NetworkingPlugin)
            .add_plugin(ServerNetworkPlugin)
            .add_network_event::<SyncShipPositionEvent>()
            .add_network_event::<SyncShipBlocksEvent>()
            .add_network_event::<SyncShipEvent>()
            .add_network_event::<PlayerMoveEvent>()
            .add_network_event::<GenericPositionSyncEvent>()
            .add_network_event::<PlayerSpawnEvent>()
            .add_network_event::<PlayerDespawnEvent>()
            .add_network_event::<BlockUpdateEvent>()
            .add_network_event::<BlockRemoveEvent>()
            .add_network_event::<LoadShipEvent>()
            .add_network_event::<EnteredShipEvent>()
            .add_network_event::<LeftShipEvent>()
            .add_network_event::<TryEnterShipEvent>()
            .add_network_event::<TryLeaveShipEvent>()
            .add_network_event::<ShipMoveEvent>()
            .add_network_event::<PlayerReadyEvent>()
            .add_network_event::<UnloadShipEvent>()
            .add_system(on_client_connect)
            .insert_resource(create_renet_server());
    }

    fn name(&self) -> &str {
        "server_network"
    }
}

fn create_renet_server() -> RenetServer {
    let current_time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();

    let rt = tokio::runtime::Runtime::new().unwrap();
    let public_ip = rt.block_on(public_ip::addr()).unwrap();
    let server_addr = SocketAddr::new(public_ip, 42069);

    //let server_addr = SocketAddr::new(local_ip().unwrap(), 42069);

    let server_config =
        ServerConfig::new(64, PROTOCOL_ID, server_addr, ServerAuthentication::Unsecure);

    let connection_config = RenetConnectionConfig::default();

    let inbound_server_addr = SocketAddr::new(local_ip().unwrap(), 42069);
    let socket = UdpSocket::bind(inbound_server_addr).unwrap();

    RenetServer::new(current_time, server_config, connection_config, socket).unwrap()
}

fn on_client_connect(
    mut server_events: EventReader<ServerEvent>,
    mut commands: Commands,
    mut network_ids: ResMut<NetworkIdMap>,
    mut player_ids: ResMut<PlayerIdMap>,
    player_query: Query<(Entity, &Name, &PlayerClientId), With<PlayerMarker>>,
    mut player_spawn_queue: ResMut<ServerMessageOutQueue<PlayerSpawnEvent>>,
    mut player_despawn_queue: ResMut<ServerMessageOutQueue<PlayerDespawnEvent>>,
    mut player_ready_queue: ResMut<ServerMessageOutQueue<PlayerReadyEvent>>,
) {
    for event in server_events.iter() {
        match event {
            ServerEvent::ClientConnected(client_id, user_data) => {
                println!("Client [{}] connected!", client_id);

                let transform = Transform::from_xyz(0., 3., 0.);

                let player_entity = commands
                    .spawn_bundle(PlayerBundle {
                        physics_object: PhysicsObjectBundle {
                            transform_bundle: TransformBundle {
                                local: transform,
                                ..default()
                            },
                            ..default()
                        },
                        name: Name::new(client_id.to_string()),
                        ..default()
                    })
                    .insert(PlayerClientId(*client_id))
                    .id();
                let network_id = network_ids.insert(player_entity);
                commands.entity(player_entity).insert(network_id);
                player_ids.insert(*client_id, player_entity);

                let mut players_online_count = 0;
                for (player_entity, player_name, player_client_id) in player_query.iter() {
                    player_spawn_queue.send(
                        client_id,
                        PlayerSpawnEvent {
                            player_entity,
                            player_name: player_name.to_string(),
                            transform,
                            player_id: player_client_id.0,
                        },
                    );

                    players_online_count += 1;
                }

                player_spawn_queue.broadcast_except(
                    client_id,
                    PlayerSpawnEvent {
                        player_entity,
                        player_name: client_id.to_string(),
                        transform,
                        player_id: *client_id,
                    },
                );

                player_ready_queue.send(
                    client_id,
                    PlayerReadyEvent {
                        own_client_it: *client_id,
                        players_online_count,
                    },
                );
            }
            ServerEvent::ClientDisconnected(client_id) => {
                println!("Client [{}] disconnected!", client_id);

                let player_entity = player_ids.from_client(*client_id).unwrap();

                commands.entity(player_entity).despawn_recursive();

                player_despawn_queue.broadcast(PlayerDespawnEvent {
                    player_entity: player_entity,
                    player_id: *client_id,
                });
            }
        }
    }
}
