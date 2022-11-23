use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket},
    time::SystemTime,
};

use bevy::prelude::Plugin;
use bevy_renet::renet::{ClientAuthentication, RenetClient, RenetConnectionConfig};
use spacegame_core::client::{AppClientNetworkTrait, ClientNetworkPlugin};

use crate::{
    events::{
        player::{PlayerDespawnEvent, PlayerMoveEvent, PlayerReadyEvent},
        ship::{
            BlockRemoveEvent, BlockUpdateEvent, EnteredShipEvent, LeftShipEvent, LoadShipEvent,
            ShipMoveEvent, TryEnterShipEvent, TryLeaveShipEvent, UnloadShipEvent,
        },
    },
    shared::{
        events::{
            generic::GenericPositionSyncEvent,
            player::PlayerSpawnEvent,
            ship::{SyncShipBlocksEvent, SyncShipEvent, SyncShipPositionEvent},
        },
        networking::plugin::NetworkingPlugin,
    },
    PROTOCOL_ID,
};

pub struct ClientNetworkingPlugin;

impl Plugin for ClientNetworkingPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugin(NetworkingPlugin)
            .add_plugin(ClientNetworkPlugin)
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
            .insert_resource(create_renet_client());
    }

    fn name(&self) -> &str {
        "client_networking"
    }
}

pub struct NetworkClient;

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
