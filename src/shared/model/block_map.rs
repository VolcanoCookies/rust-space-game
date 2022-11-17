use std::f32::consts::PI;

use bevy::prelude::{Component, Entity, EulerRot, Quat, Transform, Vec3};
use bevy::utils::hashbrown::hash_map::Iter;
use bevy::utils::HashMap;
use serde::{Deserialize, Serialize};

use super::block::BlockType;

#[derive(Hash, PartialEq, Eq, Serialize, Deserialize, Debug, Clone, Copy)]
pub struct BlockMapEntry {
    pub block_type: BlockType,
    pub block_rotation: BlockRotation,
    pub entity: Entity,
}

#[derive(Component, Serialize, Deserialize, Debug, Clone)]
pub struct BlockMap {
    map: HashMap<BlockPosition, BlockMapEntry>,
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
        match self.map.get(position) {
            Some(entry) => Some(entry.entity),
            None => None,
        }
    }

    pub fn get_entry(&self, position: &BlockPosition) -> Option<&BlockMapEntry> {
        self.map.get(position)
    }

    pub fn get_type(&self, position: &BlockPosition) -> Option<BlockType> {
        match self.map.get(position) {
            Some(entry) => Some(entry.block_type),
            None => None,
        }
    }

    pub fn set(
        &mut self,
        entity: Entity,
        block_type: BlockType,
        position: BlockPosition,
        block_rotation: BlockRotation,
    ) -> Option<BlockMapEntry> {
        let opt_old_block_entry = self.map.insert(
            position,
            BlockMapEntry {
                block_type,
                block_rotation,
                entity,
            },
        );
        if opt_old_block_entry.is_none() {
            self.block_count += 1;
        }
        opt_old_block_entry
    }

    pub fn remove(&mut self, position: &BlockPosition) -> Option<Entity> {
        let opt_old_block_entry = self.map.remove(position);
        if opt_old_block_entry.is_some() {
            self.block_count -= 1;
        }
        match opt_old_block_entry {
            Some(old_block_entry) => Some(old_block_entry.entity),
            None => None,
        }
    }

    pub fn entries(&self) -> Iter<'_, BlockPosition, BlockMapEntry> {
        self.map.iter()
    }
}

#[derive(Component, Eq, PartialEq, Hash, Clone, Copy, Serialize, Deserialize, Debug)]
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

#[derive(Component, Eq, PartialEq, Hash, Clone, Copy, Serialize, Deserialize, Debug)]
pub struct BlockRotation(u8);

impl BlockRotation {
    fn round_quat_range(v: f32) -> u8 {
        let u = (((v + PI) * 2.) / PI).round() as u8;
        if u == 4 {
            0
        } else {
            u
        }
    }

    fn round_quat_tuple(tupl: (f32, f32, f32)) -> u8 {
        Self::round_quat_range(tupl.0)
            | (Self::round_quat_range(tupl.1) >> 2)
            | (Self::round_quat_range(tupl.2) >> 4)
    }

    fn round_to_angle(u: u8) -> f32 {
        ((u as f32) - 2.) * PI / 2.
    }
}

impl Default for BlockRotation {
    fn default() -> Self {
        Quat::default().into()
    }
}

impl From<Quat> for BlockRotation {
    fn from(v: Quat) -> Self {
        Self(Self::round_quat_tuple(v.to_euler(EulerRot::XYZ)))
    }
}

impl From<BlockRotation> for Quat {
    fn from(r: BlockRotation) -> Self {
        let x = r.0 & 0b11000000;
        let y = (r.0 & 0b00110000) << 2;
        let z = (r.0 & 0b00001100) << 4;
        Quat::from_euler(
            EulerRot::XYZ,
            BlockRotation::round_to_angle(x),
            BlockRotation::round_to_angle(y),
            BlockRotation::round_to_angle(z),
        )
    }
}
