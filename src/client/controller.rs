use bevy::input::mouse::MouseMotion;
use bevy::math::{Vec2, Vec3};

use bevy::prelude::ParallelSystemDescriptorCoercion;
use bevy::prelude::{
    App, BuildChildren, Camera3d, Commands, Component, Entity, EventReader, GlobalTransform, Input,
    KeyCode, MouseButton, Parent, Query, Res, ResMut, State, SystemLabel, SystemSet, Time,
    Transform, Windows, With,
};

use bevy_debug_text_overlay::screen_print;

use bevy_rapier3d::geometry::RayIntersection;
use bevy_rapier3d::math::Real;

use bevy_rapier3d::prelude::{ExternalForce, QueryFilter, RapierContext};
use bevy_rapier3d::render::DebugRenderContext;
use iyes_loopless::prelude::{AppLooplessStateExt, IntoConditionalSystem};
use iyes_loopless::state::CurrentState;
use leafwing_input_manager::prelude::{ActionState, InputManagerPlugin, InputMap};
use leafwing_input_manager::{action_state, Actionlike, InputManagerBundle};
use spacegame_core::message::ClientMessageOutQueue;

use crate::events::ship::{
    BlockRemoveEvent, BlockUpdateEvent, EnteredShipEvent, LeftShipEvent, ShipMoveEvent,
    TryEnterShipEvent, TryLeaveShipEvent,
};
use crate::math::rotate_vec_by_quat;
use crate::model::block_map::BlockRotation;

use crate::shared::events::player::PlayerMoveEvent;
use crate::shared::model::block::BlockType;
use crate::shared::model::block_map::BlockPosition;
use crate::shared::model::ship::{Gimbal, Thrust};

use crate::shared::resources::control_input::ControlInput;
use crate::shared::resources::keybindings::Keybindings;

use super::model::character::Character;

#[derive(Component)]
pub struct Controlled;

#[derive(PartialEq, Eq, Clone, Copy, Debug, Hash)]
pub enum ControlState {
    Ship,
    Character,
    Freecam,
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
    Freecam,
}

#[derive(SystemLabel)]
enum Labels {
    Preprocess,
}

enum ControlledShip {
    Ship(Entity),
    None,
}

pub struct ControllerPlugin;

impl bevy::app::Plugin for ControllerPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ChangeControlEvent>()
            .insert_resource(ControlInput::default())
            .insert_resource(ControlledShip::None)
            .add_state(ControlState::Character)
            .add_plugin(InputManagerPlugin::<Action>::default())
            .add_startup_system(setup)
            .add_system_set(
                SystemSet::new()
                    .label(Labels::Preprocess)
                    .with_system(block_raycast)
                    .with_system(merge_inputs),
            )
            .add_system_set(
                SystemSet::on_update(ControlState::Character)
                    .after(Labels::Preprocess)
                    .with_system(character_movement)
                    .with_system(character_controls),
            )
            .add_system_set(
                SystemSet::on_update(ControlState::Ship)
                    .after(Labels::Preprocess)
                    .with_system(ship_controls),
            )
            .add_system_set(
                SystemSet::on_enter(ControlState::Character).with_system(take_character_control),
            )
            .add_system_set(SystemSet::on_enter(ControlState::Ship).with_system(take_ship_control))
            .add_system_set(
                SystemSet::on_update(ControlState::Freecam)
                    .after("preprocess")
                    .with_system(freecam_controls),
            )
            .add_system(control)
            .add_system(on_self_enter_ship)
            .add_system(on_self_leave_ship)
            .add_system(toggle_debug);
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
    ToggleFreecam,
    ResetCamera,
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
        (KeyCode::L, Action::ToggleFreecam),
        (KeyCode::R, Action::ResetCamera),
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
        QueryFilter::new().exclude_collider(character.entity),
    ) {
        if let Ok(parent) = parent_query.get(entity) {
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

fn control(
    mut commands: Commands,
    mut control_state: ResMut<State<ControlState>>,
    looking_at: Res<LookingAt>,
    control_input: Res<ControlInput>,
    action_query: Query<&ActionState<Action>>,
) {
    let action_state = action_query.single();

    if action_state.just_pressed(Action::ToggleFreecam) {
        match *control_state.current() {
            ControlState::Freecam => {
                control_state.pop();
                screen_print!("Exit freecam mode");
            }
            _ => {
                control_state.push(ControlState::Freecam);
                screen_print!("Enter freecam mode");
            }
        };
    }
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
}

// The currently controlled ship
#[derive(Component)]
pub struct ShipMarker;

fn take_character_control(
    mut commands: Commands,
    character: Res<Character>,
    camera_query: Query<(Entity, Option<&Parent>), With<Camera3d>>,
    mut transform_query: Query<&mut Transform>,
) {
    let (camera_entity, camera_parent_entity) = camera_query.single();

    if camera_parent_entity.is_some() {
        commands
            .entity(camera_parent_entity.unwrap().get())
            .remove_children(&[camera_entity]);
    }

    let mut camera_transform = transform_query.get_mut(camera_entity).unwrap();

    *camera_transform = Transform::default();
    commands.entity(character.entity).add_child(camera_entity);
}

fn character_controls(
    looking_at: Res<LookingAt>,
    state_query: Query<&ActionState<Action>>,
    transform_query: Query<&GlobalTransform>,
    block_query: Query<&BlockPosition>,
    mut block_remove_queue: ResMut<ClientMessageOutQueue<BlockRemoveEvent>>,
    mut block_update_queue: ResMut<ClientMessageOutQueue<BlockUpdateEvent>>,
    mut try_enter_ship_queue: ResMut<ClientMessageOutQueue<TryEnterShipEvent>>,
) {
    let action_state = state_query.single();

    if let LookingAt::Block(block_entity, ship_entity, intersect) = *looking_at {
        if action_state.just_pressed(Action::PlaceBlock) {
            let global_transform = transform_query.get(ship_entity).unwrap();
            let block_position = BlockPosition::rounded(
                global_transform
                    .affine()
                    .inverse()
                    .transform_point3(intersect.point + intersect.normal / 2.),
            );

            block_update_queue.send(BlockUpdateEvent {
                ship_entity,
                block_type: BlockType::Hull,
                block_position,
                block_rotation: BlockRotation::default(),
                client_id: 0,
            });
        } else if action_state.just_pressed(Action::RemoveBlock) {
            let block_position = block_query.get(block_entity).unwrap();

            block_remove_queue.send(BlockRemoveEvent {
                ship_entity,
                block_position: *block_position,
                client_id: 0,
            });
        }

        if action_state.just_pressed(Action::EnterShip) {
            try_enter_ship_queue.send(TryEnterShipEvent {
                ship_entity,
                client_id: 0,
            });
        }
    }
}

fn character_movement(
    keybindings: Res<Keybindings>,
    time: Res<Time>,
    control_input: Res<ControlInput>,
	character: Res<Character>,
	mut transform_query: Query<(&mut Transform, &GlobalTransform)>,
    mut force_query: Query<&mut ExternalForce>,
    action_query: Query<&ActionState<Action>>,
    mut player_move_queue: ResMut<ClientMessageOutQueue<PlayerMoveEvent>>,
) {
    let action_state = action_query.single();

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

    let (mut transform, _) = transform_query.get_mut(character.entity).unwrap();
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

    player_move_queue.send(PlayerMoveEvent {
        client_id: 0,
        transform: transform.clone(),
    });
}

fn ship_controls(
    time: Res<Time>,
    control_input: Res<ControlInput>,
    controlled_ship: Res<ControlledShip>,
    action_query: Query<&ActionState<Action>>,
    mut leave_ship_queue: ResMut<ClientMessageOutQueue<TryLeaveShipEvent>>,
    mut ship_move_queue: ResMut<ClientMessageOutQueue<ShipMoveEvent>>,
    mut ship_query: Query<(&mut ExternalForce, &Thrust, &Gimbal, &Transform)>,
) {
    let ship_entity = match *controlled_ship {
        ControlledShip::Ship(entity) => entity,
        ControlledShip::None => return,
    };

    let action_state = action_query.single();

    let mov_dir = action_to_axis(action_state) * Vec3::new(1., 1., -1.);
    let mut rot_dir = Vec3::ZERO;

    if action_state.pressed(Action::RotateLeft) {
        rot_dir += Vec3::new(0.0, 0.0, 1.0);
    }
    if action_state.pressed(Action::RotateRight) {
        rot_dir += Vec3::new(0.0, 0.0, -1.0);
    }

    // Double thrust when shift is pressed
    let thrust_multiplier = if action_state.pressed(Action::Boost) {
        16.0
    } else {
        8.0
    };

    rot_dir.x = -control_input.mouse_delta.y;
    rot_dir.y = -control_input.mouse_delta.x;

    let (mut force, thrust, gimbal, transform) = ship_query.single_mut();
    let dt = time.delta_seconds();
    let thrust = thrust.t * thrust_multiplier;
    let gimbal = gimbal.t * thrust_multiplier;

    force.force = rotate_vec_by_quat(mov_dir * thrust * dt, transform.rotation);
    force.torque = rotate_vec_by_quat(rot_dir * gimbal * dt, transform.rotation);

    ship_move_queue.send(ShipMoveEvent {
        ship_entity,
        force: *force,
        client_id: 0,
    });

    if action_state.just_pressed(Action::ExitShip) {
        leave_ship_queue.send(TryLeaveShipEvent {
            ship_entity,
            client_id: 0,
        });
    }
}

fn on_self_enter_ship(
    character: Res<Character>,
    mut controlled_ship: ResMut<ControlledShip>,
    mut control_state: ResMut<State<ControlState>>,
    mut events: EventReader<EnteredShipEvent>,
) {
    for event in events.iter() {
        if event.player_id == character.client_id {
            *controlled_ship = ControlledShip::Ship(event.ship_entity);
            control_state.set(ControlState::Ship);
        }
    }
}

fn on_self_leave_ship(
    character: Res<Character>,
    mut controlled_ship: ResMut<ControlledShip>,
    mut control_state: ResMut<State<ControlState>>,
    mut events: EventReader<LeftShipEvent>,
) {
    for event in events.iter() {
        if event.player_id == character.client_id {
            *controlled_ship = ControlledShip::None;
            control_state.set(ControlState::Character);
        }
    }
}

fn take_ship_control(
    mut commands: Commands,
    controlled_ship: Res<ControlledShip>,
    mut camera_query: Query<(Entity, &mut Transform, Option<&Parent>), With<Camera3d>>,
) {
    let (camera, mut transform, parent) = camera_query.single_mut();

    if parent.is_some() {
        commands
            .entity(parent.unwrap().get())
            .remove_children(&[camera]);
    }

    match *controlled_ship {
        ControlledShip::Ship(ship) => {
            commands.entity(ship).add_child(camera).insert(Controlled);
        }
        ControlledShip::None => panic!("ControlledShip is None when taking control of a ship"),
    }

    *transform = Transform::default();
}

fn freecam_controls(
    time: Res<Time>,
    action_query: Query<&ActionState<Action>>,
    mut camera_query: Query<&mut Transform, With<Camera3d>>,
) {
    let action_state = action_query.single();

    let mut camera_transform = camera_query.single_mut();

    let mut axis_input = action_to_axis(action_state);
    axis_input *= Vec3::new(1., 1., -1.);
    let speed = if action_state.pressed(Action::Boost) {
        32.
    } else {
        16.
    };

    let dt = time.delta_seconds();

    camera_transform.translation += axis_input * dt * speed;

    if action_state.just_pressed(Action::ResetCamera) {
        *camera_transform = Transform::default();
    }
}

fn action_to_axis(action_state: &ActionState<Action>) -> Vec3 {
    // Handle key input
    let mut axis_input = Vec3::ZERO;
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

    axis_input
}
