use glam::{IVec3, USizeVec3};
use indexmap::IndexMap;

use crate::{Block, CHUNK_SIZE, Chunk, WorldGenerator};

pub struct World {
    pub chunks: IndexMap<IVec3, Chunk>,
    generator: WorldGenerator,
}

impl World {
    pub fn new(generator: WorldGenerator) -> Self {
        let mut chunks = IndexMap::new();

        for x in -8..8 {
            for y in -1..1 {
                for z in -8..8 {
                    let chunk_pos = IVec3::new(x, y, z);
                    let chunk = Chunk::new(generator.generate_chunk(chunk_pos));
                    chunks.insert(chunk_pos, chunk);
                }
            }
        }

        Self { chunks, generator }
    }

    pub fn get_block(&self, world_pos: IVec3) -> Option<Block> {
        let chunk_size = CHUNK_SIZE as i32;
        let chunk_pos = world_pos.div_euclid(IVec3::splat(chunk_size));
        let chunk = self.chunks.get(&chunk_pos)?;
        let block_pos = world_pos.rem_euclid(IVec3::splat(chunk_size));
        Some(chunk.get_block(USizeVec3::new(
            block_pos.x as usize,
            block_pos.y as usize,
            block_pos.z as usize,
        )))
    }
}
