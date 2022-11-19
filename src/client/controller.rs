use std::marker::PhantomData;

use bevy::asset::Assets;

use bevy::input::mouse::MouseMotion;
use bevy::math::{EulerRot, Quat, Vec2, Vec3};

use bevy::prelude::{
    App, BuildChildren, Camera3d, Commands, Component, Entity, EventReader, EventWriter,
    GlobalTransform, Input, KeyCode, MouseButton, ParallelSystemDescriptorCoercion, Parent, Query,
    Res, ResMut, SystemSet, Time, Transform, Windows, With,
};

use bevy_debug_text_overlay::screen_print;

use bevy_rapier3d::geometry::RayIntersection;
use bevy_rapier3d::math::Real;

use bevy_rapier3d::prelude::{ExternalForce, QueryFilter, RapierContext};
use bevy_rapier3d::render::DebugRenderContext;
use bevy_renet::renet::RenetClient;
use iyes_loopless::prelude::{AppLooplessStateExt, IntoConditionalSystem};
use iyes_loopless::state::{CurrentState, NextState};
use leafwing_input_manager::prelude::{ActionState, InputManagerPlugin, InputMap};
use leafwing_input_manager::{Actionlike, InputManagerBundle};
use spacegame_core::network_id::{NetworkId, NetworkIdMap};

use crate::events::ship::{BlockRemoveEvent, BlockUpdateEvent, EnterShipEvent};
use crate::math::rotate_vec_by_quat;
use crate::model::block_map::BlockRotation;

use crate::shared::events::player::PlayerMoveEvent;
use crate::shared::model::block::BlockType;
use crate::shared::model::block_map::BlockPosition;
use crate::shared::model::ship::{Gimbal, Thrust};
use crate::shared::networking::message::{ClientMessage, NetworkMessage};

use crate::shared::resources::control_input::ControlInput;
use crate::shared::resources::keybindings::Keybindings;

use super::networking::NetworkClient;

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
            .add_plugin(InputManagerPlugin::<Action>::default())
            .add_startup_system(setup)
            .add_system(merge_inputs.label("pre_process"))
            .add_system(block_raycast.label("pre_process"))
            .add_system_set(
                SystemSet::new()
                    .after("pre_process")
                    .with_system(control)
                    .with_system(change_control)
                    .with_system(character_movement.run_in_state(ControlState::Character))
                    .with_system(character_controls.run_in_state(ControlState::Character))
                    .with_system(ship_controls.run_in_state(ControlState::Ship)),
            )
            .add_system(toggle_debug)
            .add_loopless_state(ControlState::Character);
    }

    fn name(&self) -> &str {
        "ControllerPlugin"
    }
}

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug, Component)]
enum Action {
    MoveForward,
    MoveBackward,
    MoveLeft,
    MoveRight,
    MoveUp,
    MoveDown,
    RotateLeft,
    RotateRight,
    Boost,
    PlaceBlock,
    RemoveBlock,
    EnterShip,
    ExitShip,
    ToggleDebugColliders,
    ToggleDebugPrints,
    ToggleInspector,
}

fn setup(mut commands: Commands) {
    let mut input_map = InputMap::default();
    input_map.insert_multiple([
        (KeyCode::W, Action::MoveForward),
        (KeyCode::S, Action::MoveBackward),
        (KeyCode::A, Action::MoveLeft),
        (KeyCode::D, Action::MoveRight),
        (KeyCode::Q, Action::RotateLeft),
        (KeyCode::E, Action::RotateRight),
        (KeyCode::LShift, Action::Boost),
        (KeyCode::Space, Action::MoveUp),
        (KeyCode::LControl, Action::MoveDown),
        (KeyCode::F, Action::EnterShip),
        (KeyCode::F, Action::ExitShip),
        (KeyCode::P, Action::ToggleDebugColliders),
        (KeyCode::I, Action::ToggleInspector),
    ]);

    input_map.insert_multiple([
        (MouseButton::Right, Action::PlaceBlock),
        (MouseButton::Left, Action::RemoveBlock),
    ]);

    commands.spawn_bundle(InputManagerBundle::<Action> {
        action_state: ActionState::default(),
        input_map,
    });
}

pub fn block_raycast(
    mut commands: Commands,
    character: Res<Character>,
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
        QueryFilter::new().exclude_collider(character.0),
    ) {
        if let Ok((parent)) = parent_query.get(entity) {
            commands.insert_resource(LookingAt::Block(entity, **parent, intersect));
        } else {
            commands.insert_resource(LookingAt::None);
        }
    } else {
        commands.insert_resource(LookingAt::None);
    }
}

fn merge_inputs(
    keybindings: Res<Keybindings>,
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
    camera_query: Query<Entity, With<Camera3d>>,
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

fn toggle_debug(
    mut debug_render_context: ResMut<DebugRenderContext>,
    // mut inspector_window: ResMut<InspectorWindows>,
    query: Query<&ActionState<Action>>,
) {
    let action_state = query.single();

    if action_state.just_pressed(Action::ToggleDebugColliders) {
        debug_render_context.enabled = !debug_render_context.enabled;
        screen_print!("Toggled debug lines");
    }

    //  if action_state.just_pressed(DebugAction::ToggleInspector) {
    //     let mut inspector_window_data = inspector_window.window_data_mut::<()>();
    //    inspector_window_data.visible = !inspector_window_data.visible;
    //     screen_print!("Toggled inspector");
    //   }
}

// Resource containing our own character.
pub struct Character(pub Entity);

// Our own character
#[derive(Component)]
pub struct CharacterMarker;

// The currently controlled ship
#[derive(Component)]
pub struct ShipMarker;

struct CameraController {
    pub yaw: f32,
    pub pitch: f32,
}

fn character_controls(
    looking_at: Res<LookingAt>,
    mut client: ResMut<RenetClient>,
    query: Query<&ActionState<Action>>,
    transform_query: Query<(&GlobalTransform, &NetworkId)>,
    block_query: Query<&BlockPosition>,
) {
    let action_state = query.single();

    if let LookingAt::Block(block_entity, ship_entity, intersect) = *looking_at {
        if action_state.just_pressed(Action::PlaceBlock) {
            let (global_transform, network_id) = transform_query.get(ship_entity).unwrap();
            let block_position = BlockPosition::rounded(
                global_transform
                    .affine()
                    .inverse()
                    .transform_point3(intersect.point + intersect.normal / 2.),
            );

            NetworkClient::send(
                &mut client,
                ClientMessage::UpdateBlock(BlockUpdateEvent {
                    ship_entity: network_id.into(),
                    block_type: BlockType::Hull,
                    block_position,
                    block_rotation: BlockRotation::default(),
                }),
            );
        } else if action_state.just_pressed(Action::RemoveBlock) {
            let (_, network_id) = transform_query.get(ship_entity).unwrap();
            let block_position = block_query.get(block_entity).unwrap();

            NetworkClient::send(
                &mut client,
                ClientMessage::RemoveBlock(BlockRemoveEvent {
                    ship_entity: network_id.into(),
                    block_position: *block_position,
                }),
            );
        }

        if action_state.just_pressed(Action::EnterShip) {
            let (_, network_id) = transform_query.get(ship_entity).unwrap();
            NetworkClient::send(
                &mut client,
                ClientMessage::EnterShip(EnterShipEvent {
                    ship_entity: network_id.into(),
                }),
            );
        }
    }
}

fn on_enter_ship(
    mut commands: Commands,
    character: Res<Character>,
    network_ids: Res<NetworkIdMap>,
    mut events: EventReader<EnterShipEvent>,
    controlled_query: Query<Entity, With<Controlled>>,
    camera_query: Query<Entity, With<Camera3d>>,
    mut transform_query: Query<(&mut Transform, &GlobalTransform)>,
) {
    let old_controlled_entity = controlled_query.single();
    let camera_entity = camera_query.single();

    for event in events.iter() {
        // Remove controlled from old entity and detach camera
        commands
            .entity(old_controlled_entity)
            .remove::<Controlled>()
            .remove_children(&[camera_entity]);

        // Add controlled to new entity and attach camera
        commands
            .entity(event.ship_entity)
            .insert(Controlled)
            .add_child(camera_entity);

        // Put the camera at the center of the ship
        let (mut camera_transform, _camera_global_transform) =
            transform_query.get_mut(camera_entity).unwrap();
        *camera_transform = Transform::default();

        // Convert the characters world coordinates to local ship relative coordinates
        let ship_transform = transform_query.get(event.ship_entity).unwrap().0.clone();
        let (mut character_transform, character_global_transform) =
            transform_query.get_mut(character.0).unwrap();
        character_transform.translation =
            ship_transform.translation - character_global_transform.translation();

        // Set the next state to the ship controlled
        commands.insert_resource(NextState(ControlState::Ship));
    }
}

fn character_movement(
    keybindings: Res<Keybindings>,
    time: Res<Time>,
    control_input: Res<ControlInput>,
    mut client: ResMut<RenetClient>,
    mut spawn_ship_events: EventWriter<SpawnShipEvent>,
    controlled_query: Query<Entity, With<Controlled>>,
    mut transform_query: Query<(&mut Transform, &GlobalTransform)>,
    mut force_query: Query<&mut ExternalForce>,
    action_query: Query<&ActionState<Action>>,
) {
    let action_state = action_query.single();

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
    let mut z_rot: f32 = 0.;
    if action_state.pressed(Action::MoveForward) {
        axis_input.z += 1.0;
    }
    if action_state.pressed(Action::MoveBackward) {
        axis_input.z -= 1.0;
    }
    if action_state.pressed(Action::MoveRight) {
        axis_input.x += 1.0;
    }
    if action_state.pressed(Action::MoveLeft) {
        axis_input.x -= 1.0;
    }
    if action_state.pressed(Action::MoveUp) {
        axis_input.y += 1.0;
    }
    if action_state.pressed(Action::MoveDown) {
        axis_input.y -= 1.0;
    }
    if action_state.pressed(Action::RotateLeft) {
        z_rot -= 0.0075;
    }
    if action_state.pressed(Action::RotateRight) {
        z_rot += 0.0075;
    }

    // Apply movement update
    if action_state.pressed(Action::Boost) {
        axis_input *= 24.;
    } else {
        axis_input *= 12.;
    }

    let (mut transform, _) = transform_query.get_mut(control_entity).unwrap();
    let forward = transform.forward();
    let right = transform.right();
    let up = transform.up();

    // let (mut external_force) = force_query.get_mut(control_entity).unwrap();

    // external_force.force =
    // (forward * axis_input.z * dt + right * axis_input.x * dt + up * axis_input.y * dt) * 100.0;

    transform.translation +=
        forward * axis_input.z * dt + right * axis_input.x * dt + up * axis_input.y * dt;

    if control_input.mouse_delta != Vec2::ZERO || z_rot != 0.0 {
        // Apply look update
        transform.rotate_axis(
            right,
            -control_input.mouse_delta.y * 0.5 * keybindings.character_sensitivity * dt,
        );
        transform.rotate_axis(
            up,
            -control_input.mouse_delta.x * keybindings.character_sensitivity * dt,
        );
        transform.rotate_axis(forward, z_rot);
    }

    // Send updated rotation to server
    let message = ClientMessage::PlayerMove(PlayerMoveEvent {
        client_id: client.client_id(),
        transform: transform.clone(),
    });
    client.send_message(message.channel_id(), bincode::serialize(&message).unwrap());
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
