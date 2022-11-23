use bevy::{
    prelude::{
        default, Assets, Color, Commands, DespawnRecursiveExt, EventReader, Mesh, Name, PbrBundle,
        Plugin, Query, Res, ResMut, StandardMaterial, Transform,
    },
    render::mesh,
};
use bevy_debug_text_overlay::screen_print;
use bevy_renet::renet::RenetClient;
use iyes_loopless::prelude::IntoConditionalSystem;
use spacegame_core::network_id::NetworkIdMap;

use crate::{
    client::model::character::Character,
    events::player::{PlayerDespawnEvent, PlayerMoveEvent, PlayerReadyEvent},
    networking::player_id::PlayerIdMap,
    shared::{entities::player::PlayerBundle, events::player::PlayerSpawnEvent},
};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_system(on_player_spawn.run_on_event::<PlayerSpawnEvent>())
            .add_system(on_player_despawn.run_on_event::<PlayerDespawnEvent>())
            .add_system(on_player_ready.run_on_event::<PlayerReadyEvent>())
            .add_system(on_player_move.run_on_event::<PlayerMoveEvent>());
    }

    fn name(&self) -> &str {
        "player_plugin"
    }
}

// For now ignore that we are supposed to spawn the player in after the ready event and just use it for our own client id
// Later this event will contain much other important info
fn on_player_ready(mut character: ResMut<Character>, mut events: EventReader<PlayerReadyEvent>) {
    for event in events.iter() {
        character.client_id = event.own_client_it;
        screen_print!(
            "Connected to server with {} players online",
            event.players_online_count
        );
    }
}

fn on_player_spawn(
    mut commands: Commands,
    client: Res<RenetClient>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut events: EventReader<PlayerSpawnEvent>,
) {
    for event in events.iter() {
        if event.player_id == client.client_id() {
            commands.insert_resource(Character {
                entity: event.player_entity,
                client_id: event.player_id,
                name: event.player_name.clone(),
            });
        }

        commands
            .entity(event.player_entity)
            .insert_bundle(PlayerBundle {
                name: Name::new(event.player_name.clone()),
                ..default()
            })
            .insert_bundle(PbrBundle {
                mesh: meshes.add(mesh::Mesh::from(mesh::shape::Capsule::default())),
                material: materials.add(Color::ORANGE.into()),
                ..default()
            });

        screen_print!("Spawned player {:?}", event.player_name);
    }
}

fn on_player_despawn(
    mut commands: Commands,
    mut network_ids: ResMut<NetworkIdMap>,
    mut events: EventReader<PlayerDespawnEvent>,
) {
    for event in events.iter() {
        commands.entity(event.player_entity).despawn_recursive();
        network_ids.remove(event.player_entity);

        screen_print!("Despawned player");
    }
}

fn on_player_move(
    player_id_map: Res<PlayerIdMap>,
    mut events: EventReader<PlayerMoveEvent>,
    mut query: Query<&mut Transform>,
) {
    for event in events.iter() {
        let player_entity = player_id_map.from_client(event.client_id).unwrap();
        let mut transform = query.get_mut(player_entity).unwrap();
        *transform = event.transform;
    }
}
