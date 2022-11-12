use bevy::prelude::{Input, KeyCode, MouseButton, Vec2};

pub struct ControlInput {
    pub key_input: Input<KeyCode>,
    pub mouse_input: Input<MouseButton>,
    // Mouse movement accounting for screen aspect ratio
    pub mouse_delta: Vec2,
    // Mouse movement without taking into account screen aspect ratio
    pub mouse_delta_raw: Vec2,
}

impl Default for ControlInput {
    fn default() -> Self {
        Self {
            key_input: Default::default(),
            mouse_input: Default::default(),
            mouse_delta: Default::default(),
            mouse_delta_raw: Default::default(),
        }
    }
}

impl ControlInput {
    pub fn just_pressed_key(&self, key_code: KeyCode) -> bool {
        self.key_input.just_pressed(key_code)
    }

    pub fn just_pressed_mouse(&self, mouse_button: MouseButton) -> bool {
        self.mouse_input.just_pressed(mouse_button)
    }

    pub fn pressed_key(&self, key_code: KeyCode) -> bool {
        self.key_input.pressed(key_code)
    }

    pub fn pressed_mouse(&self, mouse_button: MouseButton) -> bool {
        self.mouse_input.pressed(mouse_button)
    }
}
