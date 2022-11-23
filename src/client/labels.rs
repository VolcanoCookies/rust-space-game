use bevy::prelude::SystemLabel;

#[derive(SystemLabel)]
pub enum UpdateLabels {
    Sync,
}
