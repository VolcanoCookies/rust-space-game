use std::thread::{self, Thread};
use std::time::Duration;

use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::log::{LogPlugin, LogSettings};
use bevy::render::settings::WgpuSettings;

use bevy::winit::WinitWindows;
use bevy_framepace::{FramepacePlugin, FramepaceSettings, Limiter};
use spacegame::*;

use bevy_rapier3d::prelude::*;
use resources::keybindings::Keybindings;

use spacegame::binding::BindingPlugin;
use spacegame::model::block_map::BlockRotation;
use spacegame::server::networking::ServerNetworkingPlugin;
use spacegame::server::ship::ShipPlugin;
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
        .insert_resource(LogSettings {
            filter: "info,wgpu_core=warn,wgpu_hal=warn,spacegame=trace".into(),
            level: bevy::log::Level::TRACE,
        })
        .add_plugins(DefaultPlugins)
        .add_system_to_stage(CoreStage::Last, limit_ticks)
        .add_plugin(LogDiagnosticsPlugin::default())
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .insert_resource(BlockRegistry::new())
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_startup_system(server_setup)
        .add_system(shared::ship::despawn_ship)
        .add_plugin(ServerNetworkingPlugin)
        .add_plugin(SyncPlugin)
        .add_plugin(ShipPlugin)
        .add_plugin(BindingPlugin)
        .run();
}

fn limit_ticks(time: Res<Time>, mut _windows: NonSendMut<WinitWindows>) {
    let d = Duration::from_millis(1_000 / 64);
	if d > time.delta() {
		thread::sleep(d - time.delta());
	}
}

fn server_setup(mut commands: Commands) {
    // Spawn a single ship
    let mut block_map = BlockMap::new();
    commands
        .spawn()
        .with_children(|parent| {
            let block_position = BlockPosition::splat(0);
            let block_rotation = BlockRotation::default();

            let block_entity = parent
                .spawn_bundle(BlockBundle::new(
                    BlockType::Hull,
                    block_position,
                    block_rotation,
                ))
                .id();

            block_map.set(
                block_entity,
                BlockType::Hull,
                block_position,
                block_rotation,
            );
        })
        .insert_bundle(ShipBundle {
            block_map,
            ..default()
        });
}
