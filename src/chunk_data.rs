use std::sync::Arc;

use bevy::math::USizeVec3;
use indexmap::IndexSet;

use crate::Block;

pub const CHUNK_SIZE: usize = 32;

pub type LightData = Arc<LightDataInner>;

#[derive(Debug, Clone)]
pub struct LightDataInner {
    /// Per byte: high 4 bits = sky light (0-15), low 4 bits = block light (0-15)
    data: Vec<u8>,
}

impl Default for LightDataInner {
    fn default() -> Self {
        Self {
            data: vec![0; CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE],
        }
    }
}

impl LightDataInner {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_sky_light(&self, local_pos: USizeVec3) -> u8 {
        self.data[Self::index(local_pos)] >> 4
    }

    pub fn get_block_light(&self, local_pos: USizeVec3) -> u8 {
        self.data[Self::index(local_pos)] & 0x0F
    }

    pub fn set_sky_light(&mut self, local_pos: USizeVec3, level: u8) {
        let idx = Self::index(local_pos);
        self.data[idx] = (level << 4) | (self.data[idx] & 0x0F);
    }

    pub fn set_block_light(&mut self, local_pos: USizeVec3, level: u8) {
        let idx = Self::index(local_pos);
        self.data[idx] = (self.data[idx] & 0xF0) | (level & 0x0F);
    }

    fn index(local_pos: USizeVec3) -> usize {
        assert!(local_pos.x < CHUNK_SIZE);
        assert!(local_pos.y < CHUNK_SIZE);
        assert!(local_pos.z < CHUNK_SIZE);
        local_pos.x + local_pos.y * CHUNK_SIZE + local_pos.z * CHUNK_SIZE * CHUNK_SIZE
    }
}

pub type ChunkData = Arc<ChunkDataInner>;

#[derive(Debug, Clone)]
pub struct ChunkDataInner {
    blocks: Vec<u16>,
    // TODO: Clean up unused palette entries
    palette: IndexSet<Block>,
}

impl Default for ChunkDataInner {
    fn default() -> Self {
        Self::from_data(vec![None; CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE])
    }
}

impl ChunkDataInner {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_data(data: Vec<Option<Block>>) -> Self {
        let mut blocks = vec![0; CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE];
        let mut palette = IndexSet::new();

        for (i, block) in data.iter().enumerate() {
            if let Some(block) = block {
                palette.insert(*block);
            }

            blocks[i] = block
                .map(|block| palette.get_index_of(&block).unwrap() as u16)
                .unwrap_or(u16::MAX);
        }

        Self { blocks, palette }
    }

    pub fn get_block(&self, local_pos: USizeVec3) -> Option<Block> {
        let index = self.blocks[self.index(local_pos)];

        if index == u16::MAX {
            None
        } else {
            Some(self.palette[index as usize])
        }
    }

    pub fn set_block(&mut self, local_pos: USizeVec3, block: Option<Block>) {
        let index = self.index(local_pos);

        if let Some(block) = block {
            self.palette.insert(block);
            self.blocks[index] = self.palette.get_index_of(&block).unwrap() as u16;
        } else {
            self.blocks[index] = u16::MAX;
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = Option<Block>> {
        self.blocks.iter().map(|index| {
            if *index == u16::MAX {
                None
            } else {
                Some(self.palette[*index as usize])
            }
        })
    }

    fn index(&self, local_pos: USizeVec3) -> usize {
        assert!(local_pos.x < CHUNK_SIZE);
        assert!(local_pos.y < CHUNK_SIZE);
        assert!(local_pos.z < CHUNK_SIZE);
        local_pos.x + local_pos.y * CHUNK_SIZE + local_pos.z * CHUNK_SIZE * CHUNK_SIZE
    }
}
