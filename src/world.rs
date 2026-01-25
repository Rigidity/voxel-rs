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
        let chunk_pos = self.chunk_pos(world_pos);
        let local_pos = self.local_pos(world_pos);
        let chunk = self.chunks.get(&chunk_pos)?;
        Some(chunk.get_block(local_pos))
    }

    pub fn set_block(&mut self, world_pos: IVec3, block: Block) {
        let chunk_pos = self.chunk_pos(world_pos);
        let local_pos = self.local_pos(world_pos);
        let Some(chunk) = self.chunks.get_mut(&chunk_pos) else {
            return;
        };
        chunk.set_block(local_pos, block);
        chunk.set_dirty();

        if local_pos.x == 0
            && let Some(neighbor) = self.chunks.get_mut(&(chunk_pos - IVec3::X))
        {
            neighbor.set_dirty();
        }

        if local_pos.y == 0
            && let Some(neighbor) = self.chunks.get_mut(&(chunk_pos - IVec3::Y))
        {
            neighbor.set_dirty();
        }

        if local_pos.z == 0
            && let Some(neighbor) = self.chunks.get_mut(&(chunk_pos - IVec3::Z))
        {
            neighbor.set_dirty();
        }

        if local_pos.x == CHUNK_SIZE - 1
            && let Some(neighbor) = self.chunks.get_mut(&(chunk_pos + IVec3::X))
        {
            neighbor.set_dirty();
        }

        if local_pos.y == CHUNK_SIZE - 1
            && let Some(neighbor) = self.chunks.get_mut(&(chunk_pos + IVec3::Y))
        {
            neighbor.set_dirty();
        }

        if local_pos.z == CHUNK_SIZE - 1
            && let Some(neighbor) = self.chunks.get_mut(&(chunk_pos + IVec3::Z))
        {
            neighbor.set_dirty();
        }
    }

    pub fn chunk_pos(&self, world_pos: IVec3) -> IVec3 {
        world_pos.div_euclid(IVec3::splat(CHUNK_SIZE as i32))
    }

    pub fn local_pos(&self, world_pos: IVec3) -> USizeVec3 {
        let local_pos = world_pos.rem_euclid(IVec3::splat(CHUNK_SIZE as i32));
        USizeVec3::new(
            local_pos.x as usize,
            local_pos.y as usize,
            local_pos.z as usize,
        )
    }
}
