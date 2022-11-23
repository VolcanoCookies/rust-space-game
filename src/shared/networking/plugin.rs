use bevy::prelude::Plugin;

use super::player_id::PlayerIdMap;

pub struct NetworkingPlugin;

impl Plugin for NetworkingPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.insert_resource(PlayerIdMap::new());
    }

    fn name(&self) -> &str {
        "networking"
    }
}
