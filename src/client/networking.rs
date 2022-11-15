use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket},
    time::SystemTime,
};

use bevy::prelude::{CoreStage, EventReader, EventWriter, Plugin, Res, ResMut, SystemSet};
use bevy_renet::{
    renet::{ClientAuthentication, ReliableChannelConfig, RenetClient, RenetConnectionConfig},
    RenetClientPlugin,
};
use local_ip_address::local_ip;

use crate::{
    events::player::PlayerDespawnEvent,
    shared::{
        events::{
            generic::GenericPositionSyncEvent,
            player::PlayerSpawnEvent,
            ship::{
                AddBlockEvent, RemoveBlockEvent, SyncShipBlocksEvent, SyncShipEvent,
                SyncShipPositionEvent,
            },
        },
        networking::{message::ServerMessage, plugin::NetworkingPlugin},
    },
    PROTOCOL_ID,
};

pub struct ClientNetworkingPlugin;

impl Plugin for ClientNetworkingPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugin(NetworkingPlugin)
            .add_plugin(RenetClientPlugin)
            .add_system_to_stage(CoreStage::Update, receive_network_events)
            .insert_resource(create_renet_client());
    }

    fn name(&self) -> &str {
        "client_networking"
    }
}

fn create_renet_client() -> RenetClient {
    let current_time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();

    let socket = UdpSocket::bind("0.0.0.0:0").unwrap();
    let client_id = current_time.as_millis() as u64;

    let connection_config = RenetConnectionConfig::default();

    let server_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(85, 229, 106, 197)), 42069);
    //let server_addr = SocketAddr::new(local_ip().unwrap(), 42069);

    let authentication = ClientAuthentication::Unsecure {
        protocol_id: PROTOCOL_ID,
        client_id,
        server_addr: server_addr,
        user_data: None,
    };

    RenetClient::new(
        current_time,
        socket,
        client_id,
        connection_config,
        authentication,
    )
    .unwrap()
}

fn receive_network_events(
    mut client: ResMut<RenetClient>,
    mut sync_ship_position_events: EventWriter<SyncShipPositionEvent>,
    mut sync_ship_blocks_events: EventWriter<SyncShipBlocksEvent>,
    mut sync_ship_events: EventWriter<SyncShipEvent>,
    mut add_block_events: EventWriter<AddBlockEvent>,
    mut remove_block_events: EventWriter<RemoveBlockEvent>,
    mut generic_sync_position_events: EventWriter<GenericPositionSyncEvent>,
    mut player_spawn_events: EventWriter<PlayerSpawnEvent>,
    mut player_despawn_events: EventWriter<PlayerDespawnEvent>,
) {
    let reliable_channel_id = ReliableChannelConfig::default().channel_id;
    while let Some(message) = client.receive_message(reliable_channel_id) {
        let server_message: ServerMessage = bincode::deserialize(&message).unwrap();
        match server_message {
            ServerMessage::SyncShipPosition(event) => sync_ship_position_events.send(event),
            ServerMessage::SyncShipBlocks(event) => sync_ship_blocks_events.send(event),
            ServerMessage::AddBlock(event) => add_block_events.send(event),
            ServerMessage::RemoveBlock(event) => remove_block_events.send(event),
            ServerMessage::SyncShip(event) => sync_ship_events.send(event),
            ServerMessage::GenericPositionSync(event) => generic_sync_position_events.send(event),
            ServerMessage::PlayerSpawn(event) => player_spawn_events.send(event),
            ServerMessage::PlayerDespawn(event) => player_despawn_events.send(event),
        }
    }
}
