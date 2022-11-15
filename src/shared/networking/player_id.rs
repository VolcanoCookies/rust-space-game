use bevy::{prelude::Entity, utils::HashMap};

pub struct PlayerIdMap {
    map: HashMap<u64, Entity>,
    reverse_map: HashMap<Entity, u64>,
}

impl PlayerIdMap {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
            reverse_map: HashMap::new(),
        }
    }

    pub fn from_entity(&self, entity: Entity) -> Option<u64> {
        self.reverse_map.get(&entity).cloned()
    }

    pub fn from_client(&self, client_id: u64) -> Option<Entity> {
        self.map.get(&client_id).cloned()
    }

    pub fn insert(&mut self, client_id: u64, entity: Entity) {
        self.map.insert(client_id, entity);
        self.reverse_map.insert(entity, client_id);
    }
}
