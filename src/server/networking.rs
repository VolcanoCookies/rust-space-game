use std::{
    net::{SocketAddr, UdpSocket},
    time::SystemTime,
};

use bevy::{
    prelude::{
        default, info, warn, Commands, CoreStage, DespawnRecursiveExt, EventReader, Name, Plugin,
        Query, Res, ResMut, SystemSet, SystemStage, Transform, With,
    },
    transform::TransformBundle,
};
use bevy_rapier3d::prelude::Velocity;
use bevy_renet::{
    renet::{
        ReliableChannelConfig, RenetConnectionConfig, RenetServer, ServerAuthentication,
        ServerConfig, ServerEvent,
    },
    RenetServerPlugin,
};
use local_ip_address::local_ip;

use crate::{
    entities::physic_object::PhysicsObjectBundle,
    events::player::PlayerDespawnEvent,
    shared::{
        entities::player::{PlayerBundle, PlayerMarker},
        events::{generic::GenericPositionSyncEvent, player::PlayerSpawnEvent},
        networking::{
            message::{ClientMessage, NetworkMessage, ServerMessage},
            network_id::{NetworkId, NetworkIdMap},
            player_id::PlayerIdMap,
            plugin::NetworkingPlugin,
        },
    },
    PROTOCOL_ID,
};

pub struct ServerNetworkingPlugin;

impl Plugin for ServerNetworkingPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugin(NetworkingPlugin)
            .add_plugin(RenetServerPlugin)
            .add_system(on_client_connect)
            .add_system_set_to_stage(
                CoreStage::PreUpdate,
                SystemSet::new().with_system(receive_network_events),
            )
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
    mut server: ResMut<RenetServer>,
    mut server_events: EventReader<ServerEvent>,
    mut commands: Commands,
    mut network_ids: ResMut<NetworkIdMap>,
    mut player_ids: ResMut<PlayerIdMap>,
    mut player_query: Query<(&Name, &NetworkId), With<PlayerMarker>>,
) {
    for event in server_events.iter() {
        match event {
            ServerEvent::ClientConnected(client_id, user_data) => {
                println!("Client [{}] connected!", client_id);

                let transform = Transform::from_xyz(0., 3., 0.);

                let network_id = NetworkId::random();
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
                        network_id,
                        ..default()
                    })
                    .id();
                network_ids.insert_with_network_id(player_entity, network_id);
                player_ids.insert(*client_id, player_entity);

                for (player_name, player_network_id) in player_query.iter() {
                    let message = ServerMessage::PlayerSpawn(PlayerSpawnEvent {
                        player_network_id: *player_network_id,
                        player_name: player_name.to_string(),
                        transform,
                    });
                    server.send_message(
                        *client_id,
                        message.channel_id(),
                        bincode::serialize(&message).unwrap(),
                    );
                }

                let message = ServerMessage::PlayerSpawn(PlayerSpawnEvent {
                    player_network_id: network_id,
                    player_name: client_id.to_string(),
                    transform,
                });
                server.broadcast_message_except(
                    *client_id,
                    message.channel_id(),
                    bincode::serialize(&message).unwrap(),
                );
            }
            ServerEvent::ClientDisconnected(client_id) => {
                println!("Client [{}] disconnected!", client_id);

                let player_entity = player_ids.from_client(*client_id).unwrap();
                let network_id = network_ids.from_entity(player_entity).unwrap();

                commands.entity(player_entity).despawn_recursive();

                let message = ServerMessage::PlayerDespawn(PlayerDespawnEvent {
                    player_network_id: network_id,
                });
                server
                    .broadcast_message(message.channel_id(), bincode::serialize(&message).unwrap());
            }
        }
    }
}

fn receive_network_events(
    player_ids: Res<PlayerIdMap>,
    network_ids: Res<NetworkIdMap>,
    mut server: ResMut<RenetServer>,
) {
    let reliable_channel_id = ReliableChannelConfig::default().channel_id;
    for client_id in server.clients_id() {
        while let Some(message) = server.receive_message(client_id, reliable_channel_id) {
            let client_message: ClientMessage = bincode::deserialize(&message).unwrap();
            match client_message {
                ClientMessage::AddBlock(_) => todo!(),
                ClientMessage::RemoveBlock(_) => todo!(),
                ClientMessage::PlayerMove(player_move_event) => {
                    let player_entity = player_ids.from_client(client_id).unwrap();
                    let player_network_id = network_ids.from_entity(player_entity).unwrap();

                    let message = ServerMessage::GenericPositionSync(GenericPositionSyncEvent {
                        network_id: player_network_id,
                        transform: player_move_event.transform,
                        velocity: Velocity::zero(),
                    });

                    server.broadcast_message(
                        message.channel_id(),
                        bincode::serialize(&message).unwrap(),
                    );
                }
            }
        }
    }
}
