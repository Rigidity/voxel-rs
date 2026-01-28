use glam::USizeVec3;

use crate::Block;

pub const CHUNK_SIZE: usize = 32;

#[derive(Debug, Clone)]
pub struct ChunkData {
    blocks: Vec<Option<Block>>,
}

impl Default for ChunkData {
    fn default() -> Self {
        Self {
            blocks: vec![None; CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE],
        }
    }
}

impl ChunkData {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_block(&self, local_pos: USizeVec3) -> Option<Block> {
        self.blocks[self.index(local_pos)]
    }

    pub fn set_block(&mut self, local_pos: USizeVec3, block: Option<Block>) {
        let index = self.index(local_pos);
        self.blocks[index] = block;
    }

    fn index(&self, local_pos: USizeVec3) -> usize {
        assert!(local_pos.x < CHUNK_SIZE);
        assert!(local_pos.y < CHUNK_SIZE);
        assert!(local_pos.z < CHUNK_SIZE);
        local_pos.x + local_pos.y * CHUNK_SIZE + local_pos.z * CHUNK_SIZE * CHUNK_SIZE
    }
}
