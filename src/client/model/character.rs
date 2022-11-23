use bevy::prelude::Entity;
use spacegame_core::message::ClientId;

/// Resource containing the entity that is our own character.
pub struct Character {
    pub entity: Entity,
    pub client_id: ClientId,
    pub name: String,
}
