use std::collections::HashMap;

use glam::IVec3;

use crate::{Block, ChunkData, World};

#[derive(Debug, Clone)]
pub struct RelevantChunks {
    chunks: HashMap<IVec3, ChunkData>,
}

impl RelevantChunks {
    pub fn from_world(world: &World, center_pos: IVec3) -> Self {
        let mut chunks = HashMap::new();

        for x in -1..=1 {
            for y in -1..=1 {
                for z in -1..=1 {
                    let chunk_pos = center_pos + IVec3::new(x, y, z);
                    if let Some(chunk) = world.chunks.get(&chunk_pos) {
                        chunks.insert(chunk_pos, chunk.data.clone());
                    }
                }
            }
        }

        Self { chunks }
    }

    pub fn get_block(&self, world_pos: IVec3) -> Option<Block> {
        let chunk_pos = World::chunk_pos(world_pos);
        let local_pos = World::local_pos(world_pos);
        let chunk = self.chunks.get(&chunk_pos)?;
        chunk.get_block(local_pos)
    }
}
