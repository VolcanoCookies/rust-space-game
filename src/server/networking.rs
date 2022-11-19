use std::{
    net::{SocketAddr, UdpSocket},
    time::SystemTime,
};

use bevy::{
    ecs::event::Event,
    prelude::{
        default, Commands, CoreStage, DespawnRecursiveExt, EventReader, EventWriter, Name, Plugin,
        Query, Res, ResMut, SystemSet, Transform, With,
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
use spacegame_core::{
    network_id::{NetworkId, NetworkIdMap},
    NetworkEvent,
};

use crate::{
    entities::physic_object::PhysicsObjectBundle,
    events::{
        player::PlayerDespawnEvent,
        ship::{BlockRemoveEvent, BlockUpdateEvent, EnterShipEvent},
    },
    shared::{
        entities::player::{PlayerBundle, PlayerMarker},
        events::{generic::GenericPositionSyncEvent, player::PlayerSpawnEvent},
        networking::{
            message::{ClientMessage, NetworkMessage, ServerMessage},
            player_id::PlayerIdMap,
            plugin::NetworkingPlugin,
        },
    },
    PROTOCOL_ID,
};

pub struct NetworkServer;

impl NetworkServer {
    /// Send a message to a client over a channel.
    pub fn send_message(server: &mut RenetServer, client_id: u64, message: ServerMessage) {
        server.send_message(
            client_id,
            message.channel_id(),
            bincode::serialize(&message).unwrap(),
        );
    }

    /// Send a message to all client, except the specified one, over a channel.
    pub fn broadcast_message_except(
        server: &mut RenetServer,
        client_id: u64,
        message: ServerMessage,
    ) {
        server.broadcast_message_except(
            client_id,
            message.channel_id(),
            bincode::serialize(&message).unwrap(),
        )
    }

    /// Send a message to all client over a channel.
    pub fn broadcast_message(server: &mut RenetServer, message: ServerMessage) {
        server.broadcast_message(message.channel_id(), bincode::serialize(&message).unwrap());
    }
}

pub struct ServerNetworkingPlugin;

impl Plugin for ServerNetworkingPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        let mut server = create_renet_server();
        app.add_plugin(NetworkingPlugin)
            .add_plugin(RenetServerPlugin)
            .add_system(on_client_connect)
            .add_system_set_to_stage(
                CoreStage::PreUpdate,
                SystemSet::new().with_system(receive_network_events),
            )
            .insert_resource(server);
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
                    .id();
                let network_id = network_ids.insert(player_entity);
                commands.entity(player_entity).insert(network_id);
                player_ids.insert(*client_id, player_entity);

                for (player_name, player_network_id) in player_query.iter() {
                    let message = ServerMessage::PlayerSpawn(PlayerSpawnEvent {
                        player_entity: player_network_id.into(),
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
                    player_entity: network_id.into(),
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
                    player_entity: network_id.into(),
                });
                server
                    .broadcast_message(message.channel_id(), bincode::serialize(&message).unwrap());
            }
        }
    }
}

fn receive_network_events(
    mut commands: Commands,
    player_ids: Res<PlayerIdMap>,
    mut network_ids: ResMut<NetworkIdMap>,
    mut server: ResMut<RenetServer>,
    mut block_update_events: EventWriter<BlockUpdateEvent>,
    mut block_remove_events: EventWriter<BlockRemoveEvent>,
    mut enter_ship_events: EventWriter<EnterShipEvent>,
) {
    let reliable_channel_id = ReliableChannelConfig::default().channel_id;
    for client_id in server.clients_id() {
        while let Some(message) = server.receive_message(client_id, reliable_channel_id) {
            let client_message: ClientMessage = bincode::deserialize(&message).unwrap();
            match client_message {
                ClientMessage::UpdateBlock(event) => {
                    pass_event(
                        event,
                        &mut commands,
                        &mut network_ids,
                        &mut block_update_events,
                    );
                }
                ClientMessage::RemoveBlock(event) => {
                    pass_event(
                        event,
                        &mut commands,
                        &mut network_ids,
                        &mut block_remove_events,
                    );
                }
                ClientMessage::PlayerMove(player_move_event) => {
                    let player_entity = player_ids.from_client(client_id).unwrap();

                    let message = ServerMessage::GenericPositionSync(GenericPositionSyncEvent {
                        entity: player_entity,
                        transform: player_move_event.transform,
                        velocity: Velocity::zero(),
                    });

                    server.broadcast_message(
                        message.channel_id(),
                        bincode::serialize(&message).unwrap(),
                    );
                }
                ClientMessage::EnterShip(event) => {
                    pass_event(
                        event,
                        &mut commands,
                        &mut network_ids,
                        &mut enter_ship_events,
                    );
                }
            }
        }
    }
}

fn pass_event<T: NetworkEvent + Event>(
    mut event: T,
    commands: &mut Commands,
    network_ids: &mut NetworkIdMap,
    place_block_events: &mut EventWriter<T>,
) {
    if event.network_to_entity(commands, network_ids) {
        place_block_events.send(event);
    }
}
