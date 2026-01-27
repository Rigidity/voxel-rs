use std::collections::HashMap;

use crate::BlockKind;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockTextureKey {
    pub block_kind: BlockKind,
    pub texture_data: u64,
}

impl BlockTextureKey {
    pub fn new(block_kind: BlockKind, texture_data: u64) -> Self {
        Self {
            block_kind,
            texture_data,
        }
    }
}

#[derive(Default)]
pub struct Registry {
    block_texture_indices: HashMap<BlockTextureKey, u32>,
}

impl Registry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register_texture(
        &mut self,
        block_kind: BlockKind,
        texture_data: u64,
        texture_index: u32,
    ) {
        self.block_texture_indices.insert(
            BlockTextureKey::new(block_kind, texture_data),
            texture_index,
        );
    }

    pub fn texture_index(&self, block_kind: BlockKind, texture_data: u64) -> u32 {
        self.block_texture_indices[&BlockTextureKey::new(block_kind, texture_data)]
    }
}
