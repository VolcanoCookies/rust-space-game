use bevy::prelude::{EventReader, Plugin, Query, Res, ResMut, Transform};
use spacegame_core::message::ServerMessageOutQueue;

use crate::{networking::player_id::PlayerIdMap, shared::events::player::PlayerMoveEvent};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_system(on_player_move);
    }

    fn name(&self) -> &str {
        std::any::type_name::<Self>()
    }
}

pub fn on_player_move(
    player_ids: Res<PlayerIdMap>,
    mut events: EventReader<PlayerMoveEvent>,
    mut player_query: Query<&mut Transform>,
    mut player_move_queue: ResMut<ServerMessageOutQueue<PlayerMoveEvent>>,
) {
    for event in events.iter() {
        let player_entity = player_ids.from_client(event.client_id).unwrap();
        let mut transform = player_query.get_mut(player_entity).unwrap();
        *transform = event.transform;

        player_move_queue.broadcast_except(
            &event.client_id,
            PlayerMoveEvent {
                transform: transform.clone(),
                client_id: event.client_id,
            },
        );
    }
}
