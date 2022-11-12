use bevy::asset::Assets;

use bevy::input::mouse::MouseMotion;
use bevy::math::{EulerRot, Quat, Vec2, Vec3};

use bevy::prelude::{
    shape, App, BuildChildren, Camera, Camera3d, Commands, Component, Entity, EventReader,
    EventWriter, GlobalTransform, Input, KeyCode, Mesh, MouseButton,
    ParallelSystemDescriptorCoercion, Parent, PbrBundle, Query, Res, ResMut, SystemSet, Time,
    Transform, Windows, With,
};
use bevy::transform::TransformBundle;
use bevy::utils::default;
use bevy_rapier3d::geometry::RayIntersection;
use bevy_rapier3d::math::Real;

use bevy_rapier3d::prelude::{ExternalForce, QueryFilter, RapierContext};
use iyes_loopless::prelude::{AppLooplessStateExt, IntoConditionalSystem};
use iyes_loopless::state::{CurrentState, NextState};

use crate::math::rotate_vec_by_quat;
use crate::model::block::{BlockBundle, BlockType};
use crate::model::block_map::{BlockMap, BlockPosition};
use crate::model::ship::{Gimbal, ShipBundle, Thrust};
use crate::resources::block_registry::BlockRegistry;
use crate::resources::control_input::ControlInput;
use crate::resources::keybindings::Keybindings;

use super::placement::{BlockPlaceEvent, BlockRemoveEvent};

#[derive(Component)]
pub struct Controlled;

#[derive(PartialEq, Eq, Clone, Copy, Debug, Hash)]
pub enum ControlState {
    Ship,
    Character,
}

pub enum LookingAt {
    None,
    // Block Entity, Block Ship Entity
    Block(Entity, Entity, RayIntersection),
}

pub enum ChangeControlEvent {
    // Character entity we are taking control off
    Character(Entity),
    // Ship entity we are taking control off
    Ship(Entity),
}

pub struct SpawnShipEvent {
    pub transform: Transform,
}

pub struct ControllerPlugin;

impl bevy::app::Plugin for ControllerPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnShipEvent>()
            .add_event::<ChangeControlEvent>()
            .insert_resource(CameraController {
                pitch: 0.0,
                yaw: 0.0,
            })
            .insert_resource(ControlInput::default())
            .add_system(merge_inputs.label("pre_process"))
            .add_system(block_raycast.label("pre_process"))
            .add_system_set(
                SystemSet::new()
                    .after("pre_process")
                    .with_system(control)
                    .with_system(change_control)
                    .with_system(character_controls.run_in_state(ControlState::Character))
                    .with_system(ship_controls.run_in_state(ControlState::Ship)),
            )
            .add_loopless_state(ControlState::Character);
    }

    fn name(&self) -> &str {
        "ControllerPlugin"
    }
}

pub fn block_raycast(
    mut commands: Commands,
    rapier_context: Res<RapierContext>,
    camera_query: Query<&GlobalTransform, With<Camera3d>>,
    parent_query: Query<&Parent>,
) {
    let camera_global_transform = camera_query.single();

    if let Some((entity, intersect)) = rapier_context.cast_ray_and_get_normal(
        camera_global_transform.translation(),
        camera_global_transform.forward(),
        Real::MAX,
        true,
        QueryFilter::new(),
    ) {
        let parent = parent_query.get(entity).unwrap();
        commands.insert_resource(LookingAt::Block(entity, **parent, intersect));
    } else {
        commands.insert_resource(LookingAt::None);
    }
}

fn merge_inputs(
    keybindings: Res<Keybindings>,
    mut commands: Commands,
    key_input: Res<Input<KeyCode>>,
    mouse_input: Res<Input<MouseButton>>,
    mut windows: ResMut<Windows>,
    mut mouse_events: EventReader<MouseMotion>,
    mut control_input: ResMut<ControlInput>,
) {
    // Get aspect ratio of window
    let mut primary_window = windows.get_primary_mut().unwrap();
    let aspect_ratio = Vec2::new(primary_window.height() / primary_window.width(), 1.);

    // Handle mouse input
    let mut mouse_delta_raw = Vec2::ZERO;
    if key_input.just_released(keybindings.free_mouse) {
        primary_window.set_cursor_lock_mode(true);
        primary_window.set_cursor_visibility(false);
    }
    if key_input.just_pressed(keybindings.free_mouse) {
        primary_window.set_cursor_lock_mode(false);
        primary_window.set_cursor_visibility(true);
        let center = Vec2::new(primary_window.width() / 2., primary_window.height() / 2.);
        primary_window.set_cursor_position(center);
    }

    if !key_input.pressed(keybindings.free_mouse) && primary_window.is_focused() {
        for mouse_event in mouse_events.iter() {
            mouse_delta_raw += mouse_event.delta;
        }
    }

    // Insert the control input resource
    control_input.key_input = key_input.clone();
    control_input.mouse_input = mouse_input.clone();
    control_input.mouse_delta = mouse_delta_raw * aspect_ratio;
    control_input.mouse_delta_raw = mouse_delta_raw;
}

pub fn change_control(
    mut commands: Commands,
    character: Res<Character>,
    mut change_control_events: EventReader<ChangeControlEvent>,
    mut current_state: ResMut<CurrentState<ControlState>>,
    controlled_query: Query<Option<Entity>, With<Controlled>>,
    mut transform_query: Query<(&mut Transform, &GlobalTransform)>,
    camera_query: Query<Entity, With<Camera>>,
    parent_query: Query<&Parent>,
) {
    let opt_controlled_entity = controlled_query.single();

    let camera_entity = camera_query.single();

    for event in change_control_events.iter() {
        match event {
            ChangeControlEvent::Character(character_entity) => {
                // Gain control of a character

                // Remove Controlled component from old controlled entity
                if let Some(old_entity) = opt_controlled_entity {
                    commands.entity(old_entity).remove::<Controlled>();
                }
                commands.entity(character.0).insert(Controlled);

                let (mut character_transform, character_global_trasform) =
                    transform_query.get_mut(character.0).unwrap();
                // Remove the character from its parent, which is going to be the ship we are controlling
                if let Ok(character_parent) = parent_query.get(character.0) {
                    commands
                        .entity(character_parent.get())
                        .remove_children(&[character.0]);
                }
                // Set the characters transform to be equal its current local transform in the ship
                *character_transform = character_global_trasform.compute_transform();

                // Remove the camera from its parent, which is going to be the ship we are controlling
                if let Ok(camera_parent) = parent_query.get(camera_entity) {
                    commands
                        .entity(camera_parent.get())
                        .remove_children(&[camera_entity]);
                }

                // Set the camera to be in the characters head
                let (mut camera_transform, camera_global_transform) =
                    transform_query.get_mut(camera_entity).unwrap();
                *camera_transform = Transform::default();
                // Add the camera as a child to the character
                commands.entity(character.0).add_child(camera_entity);

                // Set the next state to be character controlled
                commands.insert_resource(NextState(ControlState::Character));
            }
            ChangeControlEvent::Ship(ship_entity) => {
                // Gain control of a ship

                // Remove Controlled component from old controlled entity
                if let Some(old_entity) = opt_controlled_entity {
                    commands.entity(old_entity).remove::<Controlled>();
                }
                // Add component
                commands.entity(*ship_entity).insert(Controlled);

                // Remove the camera from its parent, which is going to be the ship we are controlling
                if let Ok(character_parent) = parent_query.get(character.0) {
                    commands
                        .entity(character_parent.get())
                        .remove_children(&[character.0]);
                }

                // Put the camera a bit behind the center of the ship
                let (mut camera_transform, _camera_global_transform) =
                    transform_query.get_mut(camera_entity).unwrap();
                let mut camera_offset_transform =
                    Transform::from_translation(Vec3::new(0., 1.5, 2.5));
                camera_offset_transform.rotate_local_x(-25.0_f32.to_radians());
                *camera_transform = camera_offset_transform;

                // Convert the characters world coordinates to local ship relative coordinates
                let ship_transform = transform_query.get(*ship_entity).unwrap().0.clone();
                let (mut character_transform, character_global_transform) =
                    transform_query.get_mut(character.0).unwrap();
                character_transform.translation =
                    ship_transform.translation - character_global_transform.translation();

                // Add the camera as a child to the ship
                commands.entity(*ship_entity).add_child(camera_entity);
                // Add the character as a child of the ship
                commands.entity(*ship_entity).add_child(character.0);

                // Set the next state to the ship controlled
                commands.insert_resource(NextState(ControlState::Ship));
            }
        }
    }
}

pub fn control(
    keybindings: Res<Keybindings>,
    control_state: Res<CurrentState<ControlState>>,
    mut commands: Commands,
    looking_at: Res<LookingAt>,
    control_input: Res<ControlInput>,
    mut spawn_ship_events: EventWriter<SpawnShipEvent>,
) {
}

// Resource containing our own character.
pub struct Character(pub Entity);

struct CameraController {
    pub yaw: f32,
    pub pitch: f32,
}

fn character_controls(
    keybindings: Res<Keybindings>,
    time: Res<Time>,
    looking_at: Res<LookingAt>,
    control_input: Res<ControlInput>,
    mut spawn_ship_events: EventWriter<SpawnShipEvent>,
    mut change_control_events: EventWriter<ChangeControlEvent>,
    controlled_query: Query<Entity, With<Controlled>>,
    mut transform_query: Query<(&mut Transform, &GlobalTransform)>,
    mut camera_controller: ResMut<CameraController>,
    mut block_place_event: EventWriter<BlockPlaceEvent>,
    mut block_remove_event: EventWriter<BlockRemoveEvent>,
) {
    if let LookingAt::Block(block_entity, ship_entity, intersect) = *looking_at {
        if control_input.just_pressed_key(keybindings.enter_ship) {
            change_control_events.send(ChangeControlEvent::Ship(ship_entity));
            return;
        } else if control_input.just_pressed_mouse(keybindings.place) {
            // Get block pos
            let (_, ship_global_transform) = transform_query.get(ship_entity).unwrap();

            let next_block_pos = intersect.point + intersect.normal / 2.;

            let block_position = BlockPosition::rounded(
                ship_global_transform
                    .affine()
                    .inverse()
                    .transform_point3(next_block_pos),
            );

            // Place block
            block_place_event.send(BlockPlaceEvent {
                block_type: BlockType::Hull,
                block_position,
                ship_entity,
            });
        } else if control_input.just_pressed_mouse(keybindings.remove) {
            // Remove block
            block_remove_event.send(BlockRemoveEvent {
                block_entity,
                ship_entity,
            });
        }
    }

    let control_entity = controlled_query.single();
    if control_input.just_pressed_key(keybindings.spawn_ship) {
        let (_, global_transform) = transform_query.get(control_entity).unwrap();
        spawn_ship_events.send(SpawnShipEvent {
            transform: global_transform.compute_transform(),
        });
    }

    let dt = time.delta_seconds();

    // Handle key input
    let mut axis_input = Vec3::ZERO;
    if control_input.pressed_key(keybindings.forwards) {
        axis_input.z += 1.0;
    }
    if control_input.pressed_key(keybindings.backwards) {
        axis_input.z -= 1.0;
    }
    if control_input.pressed_key(keybindings.right) {
        axis_input.x += 1.0;
    }
    if control_input.pressed_key(keybindings.left) {
        axis_input.x -= 1.0;
    }
    if control_input.pressed_key(keybindings.up) {
        axis_input.y += 1.0;
    }
    if control_input.pressed_key(keybindings.down) {
        axis_input.y -= 1.0;
    }

    // Apply movement update
    if control_input.pressed_key(keybindings.boost) {
        axis_input *= 24.;
    } else {
        axis_input *= 12.;
    }

    let (mut transform, _) = transform_query.get_mut(control_entity).unwrap();
    let forward = transform.forward();
    let right = transform.right();
    let up = transform.up();
    transform.translation +=
        forward * axis_input.z * dt + right * axis_input.x * dt + up * axis_input.y * dt;

    if control_input.mouse_delta != Vec2::ZERO {
        // Apply look update
        let (pitch, yaw) = (
            (camera_controller.pitch
                - control_input.mouse_delta.y * 0.5 * keybindings.character_sensitivity * dt)
                .clamp(
                    -0.99 * std::f32::consts::FRAC_PI_2,
                    0.99 * std::f32::consts::FRAC_PI_2,
                ),
            camera_controller.yaw
                - control_input.mouse_delta.x * keybindings.character_sensitivity * dt,
        );
        transform.rotation = Quat::from_euler(EulerRot::ZYX, 0.0, yaw, pitch);
        camera_controller.pitch = pitch;
        camera_controller.yaw = yaw;
    }
}

fn ship_controls(
    keybindings: Res<Keybindings>,
    mut commands: Commands,
    time: Res<Time>,
    control_input: Res<ControlInput>,
    character: Res<Character>,
    mut change_control_events: EventWriter<ChangeControlEvent>,
    mut ship_query: Query<(&mut ExternalForce, &Thrust, &Gimbal, &Transform), With<Controlled>>,
) {
    if control_input.just_pressed_key(keybindings.leave_ship) {
        change_control_events.send(ChangeControlEvent::Character(character.0));
        return;
    }

    let mut mov_dir = Vec3::ZERO;
    let mut rot_dir = Vec3::ZERO;

    for key in control_input.key_input.get_pressed() {
        match *key {
            KeyCode::W => {
                mov_dir += Vec3::new(0.0, 0.0, -1.0);
            }
            KeyCode::S => {
                mov_dir += Vec3::new(0.0, 0.0, 1.0);
            }
            KeyCode::A => {
                mov_dir += Vec3::new(-1.0, 0.0, 0.0);
            }
            KeyCode::D => {
                mov_dir += Vec3::new(1.0, 0.0, 0.0);
            }
            KeyCode::Space => {
                mov_dir += Vec3::new(0.0, 1.0, 0.0);
            }
            KeyCode::LControl => {
                mov_dir += Vec3::new(0.0, -1.0, 0.0);
            }
            KeyCode::Q => {
                rot_dir += Vec3::new(0.0, 0.0, 1.0);
            }
            KeyCode::E => {
                rot_dir += Vec3::new(0.0, 0.0, -1.0);
            }
            _ => {}
        }
    }

    // Double thrust when shift is pressed
    let thrust_multiplier = if control_input.pressed_key(keybindings.boost) {
        2.0
    } else {
        1.0
    };

    rot_dir.x = -control_input.mouse_delta.y;
    rot_dir.y = -control_input.mouse_delta.x;

    let (mut force, thrust, gimbal, transform) = ship_query.single_mut();
    let dt = time.delta_seconds();
    let thrust = thrust.t * thrust_multiplier;
    let gimbal = gimbal.t * thrust_multiplier;

    force.force = rotate_vec_by_quat(mov_dir * thrust * dt, transform.rotation);
    force.torque = rotate_vec_by_quat(rot_dir * gimbal * dt, transform.rotation);
}

fn spawn_ship(
    mut commands: Commands,
    mut spawn_ship_events: EventReader<SpawnShipEvent>,
    block_registry: Res<BlockRegistry>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    for event in spawn_ship_events.iter() {
        // Ship
        let mut block_map = BlockMap::new();
        commands
            .spawn()
            .with_children(|parent| {
                let block_position = BlockPosition::splat(0);

                let block_entity = parent
                    .spawn_bundle(BlockBundle {
                        pbr_bundle: PbrBundle {
                            mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
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
                transform_bundle: TransformBundle {
                    local: event.transform,
                    ..default()
                },
                ..default()
            });
    }
}
