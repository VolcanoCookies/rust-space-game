use bevy::math::Vec2;

pub struct Mouse {
	pub position: Vec2,
	pub locked: bool,
}

impl Default for Mouse {
	fn default() -> Self {
		Self {
			position: Vec2::ZERO,
			locked: false
		}
	}
}