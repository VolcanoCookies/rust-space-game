use bevy::{
    prelude::{
        default, info, Assets, Color, Commands, DespawnRecursiveExt, EventReader, Mesh, Name,
        PbrBundle, Plugin, ResMut, StandardMaterial,
    },
    render::mesh,
};
use bevy_debug_text_overlay::screen_print;
use iyes_loopless::prelude::IntoConditionalSystem;

use crate::{
    events::player::PlayerDespawnEvent,
    shared::{
        entities::player::PlayerBundle, events::player::PlayerSpawnEvent,
        networking::network_id::NetworkIdMap,
    },
};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_system(on_player_spawn.run_on_event::<PlayerSpawnEvent>())
            .add_system(on_player_despawn.run_on_event::<PlayerDespawnEvent>());
    }

    fn name(&self) -> &str {
        "player_plugin"
    }
}

fn on_player_spawn(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut network_ids: ResMut<NetworkIdMap>,
    mut events: EventReader<PlayerSpawnEvent>,
) {
    for event in events.iter() {
        let entity = commands
            .spawn_bundle(PlayerBundle {
                name: Name::new(event.player_name.clone()),
                network_id: event.player_network_id,
                ..default()
            })
            .insert_bundle(PbrBundle {
                mesh: meshes.add(mesh::Mesh::from(mesh::shape::Capsule::default())),
                material: materials.add(Color::ORANGE.into()),
                ..default()
            })
            .id();
        network_ids.insert_with_network_id(entity, event.player_network_id);

        screen_print!("Spawned player");
    }
}

fn on_player_despawn(
    mut commands: Commands,
    mut network_ids: ResMut<NetworkIdMap>,
    mut events: EventReader<PlayerDespawnEvent>,
) {
    for event in events.iter() {
        if let Some(player_entity) = network_ids.from_network(event.player_network_id) {
            commands.entity(player_entity).despawn_recursive();
            network_ids.remove_network(&event.player_network_id);

            screen_print!("Despawned player");
        }
    }
}
