use bevy::prelude::Plugin;
use spacegame_core::network_id::NetworkIdMap;

use crate::{
    events::{
        player::PlayerDespawnEvent,
        ship::{BlockRemoveEvent, BlockUpdateEvent, EnterShipEvent},
    },
    shared::events::{
        generic::GenericPositionSyncEvent,
        player::{PlayerMoveEvent, PlayerSpawnEvent},
        ship::{SyncShipBlocksEvent, SyncShipEvent, SyncShipPositionEvent},
    },
};

use super::player_id::PlayerIdMap;

pub struct NetworkingPlugin;

impl Plugin for NetworkingPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_event::<SyncShipPositionEvent>()
            .add_event::<SyncShipBlocksEvent>()
            .add_event::<SyncShipEvent>()
            .add_event::<PlayerMoveEvent>()
            .add_event::<GenericPositionSyncEvent>()
            .add_event::<PlayerSpawnEvent>()
            .add_event::<PlayerDespawnEvent>()
            .add_event::<BlockUpdateEvent>()
            .add_event::<BlockRemoveEvent>()
            .add_event::<EnterShipEvent>()
            .insert_resource(NetworkIdMap::new())
            .insert_resource(PlayerIdMap::new());
    }

    fn name(&self) -> &str {
        "networking"
    }
}
