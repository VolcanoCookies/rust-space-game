use bevy::prelude::{EventReader, Query, Res, ResMut, Transform};
use bevy_renet::renet::RenetServer;

use crate::shared::{events::player::PlayerMoveEvent, networking::network_id::NetworkIdMap};

pub fn on_player_move(
    network_ids: Res<NetworkIdMap>,
    mut server: ResMut<RenetServer>,
    mut events: EventReader<PlayerMoveEvent>,
    mut player_query: Query<&mut Transform>,
) {
    for event in events.iter() {
        if let Some(player_entity) = network_ids.from_network(event.player_network_id) {
            if let Ok(mut transform) = player_query.get_mut(player_entity) {
                *transform = event.transform;
            }
        }
    }
}
