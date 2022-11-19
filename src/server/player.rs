use bevy::prelude::{EventReader, Query, Res, ResMut, Transform};
use bevy_renet::renet::RenetServer;

use crate::{networking::player_id::PlayerIdMap, shared::events::player::PlayerMoveEvent};

pub fn on_player_move(
    player_ids: Res<PlayerIdMap>,
    mut server: ResMut<RenetServer>,
    mut events: EventReader<PlayerMoveEvent>,
    mut player_query: Query<&mut Transform>,
) {
    for event in events.iter() {
        let player_entity = player_ids.from_client(event.client_id).unwrap();
        let mut transform = player_query.get_mut(player_entity).unwrap();
        *transform = event.transform;
    }
}
