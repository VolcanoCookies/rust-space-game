use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket},
    time::SystemTime,
};

use bevy::prelude::{CoreStage, EventWriter, Plugin, ResMut};
use bevy_renet::{
    renet::{ClientAuthentication, ReliableChannelConfig, RenetClient, RenetConnectionConfig},
    RenetClientPlugin,
};

use crate::{
    events::{
        player::PlayerDespawnEvent,
        ship::{BlockRemoveEvent, BlockUpdateEvent},
    },
    networking::message::{ClientMessage, NetworkMessage},
    shared::{
        events::{
            generic::GenericPositionSyncEvent,
            player::PlayerSpawnEvent,
            ship::{SyncShipBlocksEvent, SyncShipEvent, SyncShipPositionEvent},
        },
        networking::{message::ServerMessage, plugin::NetworkingPlugin},
    },
    PROTOCOL_ID,
};

pub struct ClientNetworkingPlugin;

impl Plugin for ClientNetworkingPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        let mut client = create_renet_client();
        app.add_plugin(NetworkingPlugin)
            .add_plugin(RenetClientPlugin)
            .add_system_to_stage(CoreStage::Update, receive_network_events)
            .insert_resource(client);
    }

    fn name(&self) -> &str {
        "client_networking"
    }
}

pub struct NetworkClient;

impl NetworkClient {
    pub fn send(client: &mut RenetClient, message: ClientMessage) {
        client.send_message(message.channel_id(), bincode::serialize(&message).unwrap());
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
    mut block_update_events: EventWriter<BlockUpdateEvent>,
    mut block_remove_events: EventWriter<BlockRemoveEvent>,
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
            ServerMessage::BlockUpdate(event) => block_update_events.send(event),
            ServerMessage::SyncShip(event) => sync_ship_events.send(event),
            ServerMessage::GenericPositionSync(event) => generic_sync_position_events.send(event),
            ServerMessage::PlayerSpawn(event) => player_spawn_events.send(event),
            ServerMessage::PlayerDespawn(event) => player_despawn_events.send(event),
            ServerMessage::BlockRemove(event) => block_remove_events.send(event),
            ServerMessage::EnterShip(event) => todo!(),
        }
    }
}
