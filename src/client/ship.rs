use bevy::prelude::{
    default, warn, Commands, Entity, EventReader, PbrBundle, Query, ResMut, Transform,
};
use bevy_rapier3d::prelude::Velocity;
use spacegame_core::network_id::NetworkIdMap;

use crate::{
    model::{
        block::{BlockBundle, BlockType},
        block_map::BlockPosition,
    },
    resources::block_registry::BlockRegistry,
    shared::events::ship::SyncShipPositionEvent,
};

pub fn sync_ship_position(
    mut commands: Commands,
    mut network_ids: ResMut<NetworkIdMap>,
    mut events: EventReader<SyncShipPositionEvent>,
    mut ship_query: Query<(&mut Transform, &mut Velocity)>,
) {
    for event in events.iter() {
        // Sync existing ship
        if let Ok((mut ship_transform, mut ship_velocity)) = ship_query.get_mut(event.ship_entity) {
            *ship_transform = event.transform;
            *ship_velocity = event.velocity;
        }
    }
}

fn spawn_block(
    commands: &mut Commands,
    block_registry: &BlockRegistry,
    block_position: BlockPosition,
    block_type: BlockType,
) -> Entity {
    commands
        .spawn_bundle(BlockBundle {
            block_type,
            block_position,
            pbr_bundle: PbrBundle {
                transform: block_position.into(),
                mesh: block_registry.get_mesh(block_type),
                material: block_registry.get_material(block_type),
                ..default()
            },
            ..default()
        })
        .id()
}
