use bevy::asset::Handle;
use bevy::prelude::{Mesh, StandardMaterial};
use bevy::utils::HashMap;

use crate::model::block::BlockType;

pub struct BlockRegistry {
    material_map: HashMap<BlockType, Handle<StandardMaterial>>,
    mesh_map: HashMap<BlockType, Handle<Mesh>>,
}

impl BlockRegistry {
    pub fn new() -> Self {
        Self {
            material_map: HashMap::new(),
            mesh_map: HashMap::new(),
        }
    }

    pub fn register_material(
        &mut self,
        block_type: BlockType,
        material_handle: Handle<StandardMaterial>,
    ) {
        self.material_map.insert(block_type, material_handle);
    }

    pub fn register_mesh(&mut self, block_type: BlockType, mesh_handle: Handle<Mesh>) {
        self.mesh_map.insert(block_type, mesh_handle);
    }

    pub fn get_material(&self, block_type: BlockType) -> Handle<StandardMaterial> {
        self.material_map.get(&block_type).cloned().unwrap()
    }

    pub fn get_mesh(&self, block_type: BlockType) -> Handle<Mesh> {
        self.mesh_map.get(&block_type).cloned().unwrap()
    }
}
