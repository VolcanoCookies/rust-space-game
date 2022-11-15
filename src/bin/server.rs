use bevy::render::settings::WgpuSettings;

use spacegame::*;

use bevy_rapier3d::prelude::*;
use resources::keybindings::Keybindings;

use shared::placement::PlacementPlugin;
use spacegame::server::networking::ServerNetworkingPlugin;
use spacegame::server::sync::SyncPlugin;

use crate::model::block::{BlockBundle, BlockType};
use crate::model::block_map::{BlockMap, BlockPosition};
use crate::model::ship::ShipBundle;
use crate::resources::block_registry::BlockRegistry;

use spacegame::server::*;

fn main() {
    App::new()
        .insert_resource(RapierConfiguration {
            gravity: Vect::ZERO,
            ..default()
        })
        .insert_resource(Keybindings::default())
        .insert_resource(WgpuSettings {
            backends: None,
            ..default()
        })
        .add_plugins(DefaultPlugins)
        .insert_resource(BlockRegistry::new())
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_startup_system(server_setup)
        .add_plugin(PlacementPlugin)
        .add_system(shared::ship::despawn_ship)
        .add_plugin(ServerNetworkingPlugin)
        .add_plugin(SyncPlugin)
        .run();
}

fn server_setup(mut commands: Commands, mut block_registry: ResMut<BlockRegistry>) {
    // Ship
    let mut block_map = BlockMap::new();
    commands
        .spawn()
        .with_children(|parent| {
            let block_position = BlockPosition::splat(0);

            let block_entity = parent
                .spawn_bundle(BlockBundle {
                    pbr_bundle: PbrBundle {
                        transform: Transform::from_xyz(0., 0., 0.),
                        ..default()
                    },
                    block_position,
                    block_type: BlockType::Hull,
                    ..default()
                })
                .id();

            block_map.set(block_position, BlockType::Hull, block_entity);
        })
        .insert_bundle(ShipBundle {
            block_map,
            ..default()
        });
}
