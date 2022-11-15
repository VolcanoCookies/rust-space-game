use bevy::prelude::{Commands, DespawnRecursiveExt, Entity, Query};

use crate::model::block_map::BlockMap;

pub fn despawn_ship(mut commands: Commands, mut query: Query<(Entity, &BlockMap)>) {
    // Despawn ship if it has no blocks
    for (entity, block_map) in query.iter_mut() {
        if block_map.block_count == 0 {
            commands.entity(entity).despawn_recursive();
        }
    }
}
