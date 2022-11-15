use std::ops::{Index, IndexMut};

use bevy::{
    prelude::{Component, Entity},
    utils::{default, HashMap},
};
use serde::{Deserialize, Serialize};

#[derive(Component, Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct NetworkId(pub u64);

impl NetworkId {
    pub fn random() -> Self {
        Self(fastrand::u64(u64::MIN..u64::MAX))
    }
}

pub struct NetworkIdMap {
    map: HashMap<NetworkId, Entity>,
    reverse_map: HashMap<Entity, NetworkId>,
}

impl NetworkIdMap {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
            reverse_map: HashMap::new(),
        }
    }

    pub fn from_network(&self, network_id: NetworkId) -> Option<Entity> {
        self.map.get(&network_id).cloned()
    }

    pub fn from_entity(&self, entity: Entity) -> Option<NetworkId> {
        self.reverse_map.get(&entity).cloned()
    }

    pub fn get(&mut self, entity: Entity) -> NetworkId {
        match self.from_entity(entity) {
            Some(network_id) => network_id,
            None => self.insert_no_check(entity),
        }
    }

    pub fn insert_with_network_id(&mut self, entity: Entity, network_id: NetworkId) {
        self.map.insert(network_id, entity);
        self.reverse_map.insert(entity, network_id);
    }

    fn insert_no_check(&mut self, entity: Entity) -> NetworkId {
        let network_id = NetworkId::random();
        self.map.insert_unique_unchecked(network_id, entity);
        self.reverse_map.insert_unique_unchecked(entity, network_id);
        network_id
    }

    pub fn remove_entity(&mut self, entity: &Entity) {
        if let Some(network_id) = self.reverse_map.remove(entity) {
            self.map.remove(&network_id);
        }
    }

    pub fn remove_network(&mut self, network_id: &NetworkId) {
        if let Some(entity) = self.map.remove(network_id) {
            self.reverse_map.remove(&entity);
        }
    }
}
