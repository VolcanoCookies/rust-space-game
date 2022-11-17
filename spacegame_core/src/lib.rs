use bevy::{
    prelude::{Component, Entity, EventReader, Plugin},
    utils::HashMap,
};
use serde::{Deserialize, Serialize};

pub struct NetworkIdPlugin;

impl Plugin for NetworkIdPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.insert_resource(NetworkIdMap::new());
    }

    fn name(&self) -> &str {
        "renet_network_id_plugin"
    }
}

impl NetworkIdPlugin {
    fn before_send(mut events: EventReader<NetworkEvent>) {}
}

pub struct NetworkIdMap {
    map: HashMap<Entity, NetworkId>,
    reverse_map: HashMap<NetworkId, Entity>,
}

impl NetworkIdMap {
    fn new() -> Self {
        Self {
            map: HashMap::new(),
            reverse_map: HashMap::new(),
        }
    }

    fn from_network(&self, network_id: NetworkId) -> Option<Entity> {
        self.reverse_map.get(&network_id).copied()
    }

    fn from_entity(&self, entity: Entity) -> Option<NetworkId> {
        self.map.get(&entity).copied()
    }

    fn insert(&mut self, entity: Entity) -> NetworkId {
        match self.map.get(&entity) {
            Some(network_id) => *network_id,
            None => {
                let mut network_id = NetworkId::random();
                while (self.map.contains_key(&network_id)) {
                    network_id = NetworkId::random();
                }
                self.map.insert_unique_unchecked(entity, network_id);
                self.reverse_map.insert_unique_unchecked(network_id, entity);
                network_id
            }
        }
    }
}

#[derive(Component, Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct NetworkId(u64);

impl NetworkId {
    fn random() -> Self {
        Self(fastrand::u64(u64::MIN..u64::MAX))
    }
}

impl From<Entity> for NetworkId {
    fn from(e: Entity) -> Self {
        Self(e.to_bits())
    }
}

impl Into<Entity> for NetworkId {
    fn into(self) -> Entity {
        Entity::from_bits(self.0)
    }
}

pub(crate) enum OnUnknownId {
    Drop,
    CreateEntity,
}

pub(crate) trait NetworkEvent {
    const CHANNEL_ID: u64;
    const ORDERED: bool;
    const OnUnknownId: OnUnknownId;

    fn entity_to_network();
    fn network_to_entity();
}
