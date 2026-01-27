use glam::USizeVec3;

use crate::{Block, ChunkData};

pub const CHUNK_SIZE: usize = 32;

#[derive(Debug, Clone)]
pub struct Chunk {
    pub data: ChunkData,
    is_dirty: bool,
}

impl Chunk {
    pub fn new(data: ChunkData) -> Self {
        Self {
            data,
            is_dirty: true,
        }
    }

    pub fn get_block(&self, local_pos: USizeVec3) -> Option<Block> {
        self.data.get_block(local_pos)
    }

    pub fn set_block(&mut self, local_pos: USizeVec3, block: Option<Block>) {
        self.data.set_block(local_pos, block);
        self.is_dirty = true;
    }

    pub fn is_dirty(&self) -> bool {
        self.is_dirty
    }

    pub fn clear_dirty(&mut self) {
        self.is_dirty = false;
    }

    pub fn set_dirty(&mut self) {
        self.is_dirty = true;
    }
}
