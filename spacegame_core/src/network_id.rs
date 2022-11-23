use bevy::{
    prelude::{Component, Entity},
    utils::HashMap,
};
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub struct NetworkIdMap {
    map: HashMap<Entity, NetworkId>,
    reverse_map: HashMap<NetworkId, Entity>,
}

impl NetworkIdMap {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
            reverse_map: HashMap::new(),
        }
    }

    pub fn from_network(&self, network_id: NetworkId) -> Option<Entity> {
        self.reverse_map.get(&network_id).copied()
    }

    pub fn from_entity(&self, entity: Entity) -> Option<NetworkId> {
        self.map.get(&entity).copied()
    }

    pub fn insert(&mut self, entity: Entity) -> NetworkId {
        match self.map.get(&entity) {
            Some(network_id) => *network_id,
            None => {
                let mut network_id = NetworkId::random();
                while self.reverse_map.contains_key(&network_id) {
                    network_id = NetworkId::random();
                }

                self.map.insert_unique_unchecked(entity, network_id);
                self.reverse_map.insert_unique_unchecked(network_id, entity);
                network_id
            }
        }
    }

    pub fn insert_with_id(&mut self, entity: Entity, network_id: NetworkId) {
        match self.reverse_map.get(&network_id) {
            Some(_) => panic!("Entity already has network id"),
            None => {
                self.map.insert_unique_unchecked(entity, network_id);
                self.reverse_map.insert_unique_unchecked(network_id, entity);
            }
        }
    }

    pub fn remove(&mut self, entity: Entity) {
        if let Some(network_id) = self.map.remove(&entity) {
            self.reverse_map.remove(&network_id);
        }
    }
}

#[derive(Component, Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct NetworkId(u32);

impl NetworkId {
    fn random() -> Self {
        Self(fastrand::u32(u32::MIN..u32::MAX))
    }
}

impl From<Entity> for NetworkId {
    fn from(e: Entity) -> Self {
        Self(e.id())
    }
}

impl Into<Entity> for NetworkId {
    fn into(self) -> Entity {
        Entity::from_raw(self.0)
    }
}

impl Into<Entity> for &NetworkId {
    fn into(self) -> Entity {
        Entity::from_raw(self.0)
    }
}
