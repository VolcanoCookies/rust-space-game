use bevy::prelude::{Component, Entity, Transform, Vec3};
use bevy::utils::HashMap;

#[derive(Component)]
pub struct BlockMap {
    map: HashMap<BlockPosition, Entity>,
    pub block_count: i32,
}

impl BlockMap {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
            block_count: 0,
        }
    }

    pub fn get(&self, position: &BlockPosition) -> Option<Entity> {
        self.map.get(position).cloned()
    }

    pub fn set(&mut self, position: BlockPosition, entity: Entity) -> Option<Entity> {
        let opt_old_block = self.map.insert(position, entity);
        if opt_old_block.is_none() {
            self.block_count += 1;
        }
        opt_old_block
    }

    pub fn remove(&mut self, position: &BlockPosition) -> Option<Entity> {
        let opt_old_block = self.map.remove(position);
        if opt_old_block.is_some() {
            self.block_count -= 1;
        }
        opt_old_block
    }
}

#[derive(Component, Eq, PartialEq, Hash, Clone, Copy)]
pub struct BlockPosition {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl Default for BlockPosition {
    fn default() -> Self {
        BlockPosition::splat(0)
    }
}

impl BlockPosition {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    pub fn splat(v: i32) -> Self {
        Self { x: v, y: v, z: v }
    }

    pub fn rounded(v: Vec3) -> Self {
        Self {
            x: v.x.round() as i32,
            y: v.y.round() as i32,
            z: v.z.round() as i32,
        }
    }
}

impl From<Vec3> for BlockPosition {
    fn from(vec: Vec3) -> Self {
        Self {
            x: vec.x as i32,
            y: vec.y as i32,
            z: vec.z as i32,
        }
    }
}

impl Into<Vec3> for BlockPosition {
    fn into(self) -> Vec3 {
        Vec3 {
            x: self.x as f32,
            y: self.y as f32,
            z: self.z as f32,
        }
    }
}

impl Into<Transform> for BlockPosition {
    fn into(self) -> Transform {
        Transform::from_translation(self.into())
    }
}
