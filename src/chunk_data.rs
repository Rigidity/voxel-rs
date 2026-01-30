use std::sync::Arc;

use bevy::math::USizeVec3;

use crate::Block;

pub const CHUNK_SIZE: usize = 32;

#[derive(Debug, Clone)]
pub struct ChunkData {
    blocks: Arc<Vec<Option<Block>>>,
}

impl Default for ChunkData {
    fn default() -> Self {
        Self::from_data(vec![None; CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE])
    }
}

impl ChunkData {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_data(data: Vec<Option<Block>>) -> Self {
        Self {
            blocks: Arc::new(data),
        }
    }

    pub fn clone_data(&self) -> Vec<Option<Block>> {
        self.blocks.as_ref().clone()
    }

    pub fn get_block(&self, local_pos: USizeVec3) -> Option<Block> {
        self.blocks[self.index(local_pos)]
    }

    pub fn set_block(&mut self, local_pos: USizeVec3, block: Option<Block>) {
        let index = self.index(local_pos);
        Arc::make_mut(&mut self.blocks)[index] = block;
    }

    fn index(&self, local_pos: USizeVec3) -> usize {
        assert!(local_pos.x < CHUNK_SIZE);
        assert!(local_pos.y < CHUNK_SIZE);
        assert!(local_pos.z < CHUNK_SIZE);
        local_pos.x + local_pos.y * CHUNK_SIZE + local_pos.z * CHUNK_SIZE * CHUNK_SIZE
    }
}
