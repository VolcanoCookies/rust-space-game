use bevy::input::keyboard::KeyboardInput;
use bevy::math::{EulerRot, Quat, Vec2, Vec3};
use bevy::prelude::{
    Camera3d, CursorMoved, EventReader, Input, KeyCode, MouseButton, Query, Res, ResMut, Transform,
    Windows, With,
};
use bevy::window::Window;

use crate::math::rotate_vec_by_quat;
use crate::resources::mouse::Mouse;
