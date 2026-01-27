use std::collections::HashMap;

use crate::BlockType;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockTextureKey {
    pub block_id: u16,
    pub texture_data: u64,
}

impl BlockTextureKey {
    pub fn new(block_id: u16, texture_data: u64) -> Self {
        Self {
            block_id,
            texture_data,
        }
    }
}

#[derive(Default)]
pub struct Registry {
    block_types: Vec<&'static dyn BlockType>,
    block_ids: HashMap<&'static str, u16>,
    block_texture_indices: HashMap<BlockTextureKey, u32>,
}

impl Registry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register_block_type(&mut self, block_type: &'static dyn BlockType) {
        let name = block_type.base_name();
        let id = self.block_types.len() as u16;
        self.block_types.push(block_type);
        self.block_ids.insert(name, id);
    }

    pub fn block_id(&self, name: &str) -> u16 {
        self.block_ids[name]
    }

    pub fn block_type(&self, id: u16) -> &'static dyn BlockType {
        self.block_types[id as usize]
    }

    pub fn block_ids(&self) -> impl Iterator<Item = u16> {
        self.block_ids.values().copied()
    }

    pub fn register_texture(&mut self, block_id: u16, texture_data: u64, texture_index: u32) {
        self.block_texture_indices
            .insert(BlockTextureKey::new(block_id, texture_data), texture_index);
    }

    pub fn texture_index(&self, block_id: u16, texture_data: u64) -> u32 {
        self.block_texture_indices[&BlockTextureKey::new(block_id, texture_data)]
    }
}
