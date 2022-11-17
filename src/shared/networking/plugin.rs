use bevy::prelude::Plugin;

use crate::{
    events::{
        player::PlayerDespawnEvent,
        ship::{BlockRemoveEvent, BlockUpdateEvent},
    },
    shared::events::{
        generic::GenericPositionSyncEvent,
        player::{PlayerMoveEvent, PlayerSpawnEvent},
        ship::{
            PlaceBlockEvent, RemoveBlockEvent, SyncShipBlocksEvent, SyncShipEvent,
            SyncShipPositionEvent,
        },
    },
};

use super::{network_id::NetworkIdMap, player_id::PlayerIdMap};

pub struct NetworkingPlugin;

impl Plugin for NetworkingPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_event::<SyncShipPositionEvent>()
            .add_event::<SyncShipBlocksEvent>()
            .add_event::<SyncShipEvent>()
            .add_event::<PlaceBlockEvent>()
            .add_event::<RemoveBlockEvent>()
            .add_event::<PlayerMoveEvent>()
            .add_event::<GenericPositionSyncEvent>()
            .add_event::<PlayerSpawnEvent>()
            .add_event::<PlayerDespawnEvent>()
            .add_event::<BlockUpdateEvent>()
            .add_event::<BlockRemoveEvent>()
            .insert_resource(NetworkIdMap::new())
            .insert_resource(PlayerIdMap::new());
    }

    fn name(&self) -> &str {
        "networking"
    }
}
