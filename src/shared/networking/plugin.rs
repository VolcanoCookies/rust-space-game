use bevy::prelude::Plugin;

use crate::{
    events::player::PlayerDespawnEvent,
    shared::events::{
        generic::GenericPositionSyncEvent,
        player::{PlayerMoveEvent, PlayerSpawnEvent},
        ship::{
            AddBlockEvent, RemoveBlockEvent, SyncShipBlocksEvent, SyncShipEvent,
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
            .add_event::<AddBlockEvent>()
            .add_event::<RemoveBlockEvent>()
            .add_event::<PlayerMoveEvent>()
            .add_event::<GenericPositionSyncEvent>()
            .add_event::<PlayerSpawnEvent>()
            .add_event::<PlayerDespawnEvent>()
            .insert_resource(NetworkIdMap::new())
            .insert_resource(PlayerIdMap::new());
    }

    fn name(&self) -> &str {
        "networking"
    }
}
