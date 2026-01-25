use glam::USizeVec3;

use crate::{Block, CHUNK_SIZE};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChunkData {
    blocks: [Block; CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE],
}

impl Default for ChunkData {
    fn default() -> Self {
        Self {
            blocks: [Block::Air; CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE],
        }
    }
}

impl ChunkData {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_block(&self, local_pos: USizeVec3) -> Block {
        self.blocks[self.index(local_pos)]
    }

    pub fn set_block(&mut self, local_pos: USizeVec3, block: Block) {
        self.blocks[self.index(local_pos)] = block;
    }

    fn index(&self, local_pos: USizeVec3) -> usize {
        assert!(local_pos.x < CHUNK_SIZE);
        assert!(local_pos.y < CHUNK_SIZE);
        assert!(local_pos.z < CHUNK_SIZE);
        local_pos.x + local_pos.y * CHUNK_SIZE + local_pos.z * CHUNK_SIZE * CHUNK_SIZE
    }
}
