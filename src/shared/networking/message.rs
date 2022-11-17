use bevy_renet::renet::ReliableChannelConfig;
use serde::Deserialize;
use serde::Serialize;

use crate::events::player::PlayerDespawnEvent;
use crate::events::ship::BlockRemoveEvent;
use crate::events::ship::BlockUpdateEvent;
use crate::events::ship::EnterShipEvent;
use crate::shared::events::generic::GenericPositionSyncEvent;
use crate::shared::events::player::PlayerMoveEvent;
use crate::shared::events::player::PlayerSpawnEvent;
use crate::shared::events::ship::SyncShipEvent;
use crate::shared::events::ship::{
    PlaceBlockEvent, RemoveBlockEvent, SyncShipBlocksEvent, SyncShipPositionEvent,
};

pub trait NetworkMessage {
    fn channel_id(&self) -> u8;
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ServerMessage {
    SyncShipPosition(SyncShipPositionEvent),
    SyncShipBlocks(SyncShipBlocksEvent),
    // To ensure both of the events arrive at the same time
    SyncShip(SyncShipEvent),
    BlockUpdate(BlockUpdateEvent),
    BlockRemove(BlockRemoveEvent),
    GenericPositionSync(GenericPositionSyncEvent),
    PlayerSpawn(PlayerSpawnEvent),
    PlayerDespawn(PlayerDespawnEvent),
    EnterShip(EnterShipEvent),
}

impl NetworkMessage for ServerMessage {
    fn channel_id(&self) -> u8 {
        let reliable_channel_id = ReliableChannelConfig::default().channel_id;
        match self {
            _ => reliable_channel_id,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ClientMessage {
    PlaceBlock(PlaceBlockEvent),
    RemoveBlock(RemoveBlockEvent),
    PlayerMove(PlayerMoveEvent),
    EnterShip(EnterShipEvent),
}

impl NetworkMessage for ClientMessage {
    fn channel_id(&self) -> u8 {
        let reliable_channel_id = ReliableChannelConfig::default().channel_id;
        match self {
            _ => reliable_channel_id,
        }
    }
}
