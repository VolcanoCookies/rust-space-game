use bevy::asset::LoadState;

use bevy::prelude::*;

use bevy::render::texture::ImageSettings;
use bevy_inspector_egui::{RegisterInspectable, WorldInspectorPlugin};
use bevy_prototype_debug_lines::DebugLinesPlugin;
use bevy_rapier3d::prelude::*;
use resources::keybindings::Keybindings;
use system::controller::{Character, Controlled, ControllerPlugin};
use system::placement::PlacementPlugin;

use crate::model::block::{BlockBundle, BlockType};
use crate::model::block_map::{BlockMap, BlockPosition};
use crate::model::ship::ShipBundle;
use crate::resources::block_registry::BlockRegistry;
use crate::resources::mouse::Mouse;
use crate::resources::skybox::{asset_loaded, Cubemap, CubemapMaterial};
use crate::system::controller::LookingAt;

mod math;
mod model;
mod resources;
mod system;

fn main() {
    App::new()
        .insert_resource(ImageSettings::default_nearest())
        .insert_resource(RapierConfiguration {
            gravity: Vect::ZERO,
            ..default()
        })
        .insert_resource(WindowDescriptor {
            cursor_locked: true,
            cursor_visible: false,
            title: "Rust Space Game".into(),
            ..default()
        })
        .insert_resource(Keybindings::default())
        .add_plugins(DefaultPlugins)
        .add_plugin(WorldInspectorPlugin::new())
        .add_plugin(DebugLinesPlugin::with_depth_test(true))
        .insert_resource(BlockRegistry::new())
        .insert_resource(Mouse::default())
        .insert_resource(LookingAt::None)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(RapierDebugRenderPlugin::default())
        .add_startup_system(setup)
        .add_plugin(ControllerPlugin)
        .add_plugin(PlacementPlugin)
        .add_system(system::highlight::highlight_mouse_pressed)
        .add_system(system::ship::despawn_ship)
        .add_plugin(MaterialPlugin::<CubemapMaterial>::default())
        .add_system(asset_loaded)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
    mut block_registry: ResMut<BlockRegistry>,
) {
    let hull_handle = asset_server.load("hull.png");

    // Register block types
    let hull_material = StandardMaterial {
        base_color_texture: Some(hull_handle),
        ..default()
    };
    let hull_material_handle = materials.add(hull_material);
    block_registry.register_material(BlockType::Hull, hull_material_handle);
    let hull_mesh_handle = meshes.add(Mesh::from(shape::Cube { size: 1.0 }));
    block_registry.register_mesh(BlockType::Hull, hull_mesh_handle);

    // Plane
    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Plane { size: 100. })),
        material: materials.add(Color::rgb(0.5, 0.5, 0.5).into()),
        transform: Transform::from_translation(Vec3::new(0., -2.5, 0.)),
        ..Default::default()
    });

    // Light
    commands.spawn_bundle(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });

    // Ship
    let mut block_map = BlockMap::new();
    commands
        .spawn()
        .with_children(|parent| {
            let block_position = BlockPosition::splat(0);

            let block_entity = parent
                .spawn_bundle(BlockBundle {
                    pbr_bundle: PbrBundle {
                        mesh: block_registry.get_mesh(BlockType::Hull),
                        material: block_registry.get_material(BlockType::Hull),
                        transform: Transform::from_xyz(0., 0., 0.),
                        ..default()
                    },
                    block_position,
                    ..default()
                })
                .id();

            block_map.set(block_position, block_entity);
        })
        .insert_bundle(ShipBundle {
            block_map,
            ..default()
        });

    // Camera
    let camera_entity = commands.spawn_bundle(Camera3dBundle::default()).id();

    // Character
    let character_entity = commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Capsule::default())),
            material: materials.add(Color::ALICE_BLUE.into()),
            ..default()
        })
        .insert(Controlled)
        .add_child(camera_entity)
        .id();

    commands.insert_resource(Character(character_entity));

    // Crosshair
    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                position_type: PositionType::Absolute,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            color: Color::NONE.into(),
            ..default()
        })
        .with_children(|parent| {
            parent.spawn_bundle(NodeBundle {
                style: Style {
                    size: Size::new(Val::Px(32.0), Val::Px(32.0)),
                    ..default()
                },
                color: Color::rgb(0.65, 0.65, 0.65).into(),
                image: UiImage::from(asset_server.load("crosshair.png")),
                ..default()
            });
        });

    // Skybox
    let skybox_handle = asset_server.load("skybox_big.png");

    // Insert skybox resource
    commands.insert_resource(Cubemap {
        is_loaded: false,
        index: 0,
        image_handle: skybox_handle,
    });
}
