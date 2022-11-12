use bevy::prelude::{KeyCode, MouseButton};

pub struct Keybindings {
    pub forwards: KeyCode,
    pub backwards: KeyCode,
    pub left: KeyCode,
    pub right: KeyCode,
    pub up: KeyCode,
    pub down: KeyCode,
    pub rotate_clockwise: KeyCode,
    pub rotate_counter_clockwise: KeyCode,
    pub enter_ship: KeyCode,
    pub leave_ship: KeyCode,
    pub boost: KeyCode,
    pub free_mouse: KeyCode,
    pub character_sensitivity: f32,
    pub ship_sensitivity: f32,
    pub place: MouseButton,
    pub remove: MouseButton,
    pub spawn_ship: KeyCode,
}

impl Default for Keybindings {
    fn default() -> Self {
        Self {
            forwards: KeyCode::W,
            backwards: KeyCode::S,
            left: KeyCode::A,
            right: KeyCode::D,
            up: KeyCode::Space,
            down: KeyCode::LControl,
            rotate_clockwise: KeyCode::E,
            rotate_counter_clockwise: KeyCode::Q,
            enter_ship: KeyCode::F,
            leave_ship: KeyCode::F,
            boost: KeyCode::LShift,
            free_mouse: KeyCode::LAlt,
            character_sensitivity: 0.25,
            ship_sensitivity: 0.25,
            place: MouseButton::Left,
            remove: MouseButton::Right,
            spawn_ship: KeyCode::B,
        }
    }
}
