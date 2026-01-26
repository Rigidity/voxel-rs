use glam::{IVec3, USizeVec3, Vec3};
use noise::Perlin;

use crate::{Block, CHUNK_SIZE, ChunkData};

#[derive(Debug, Clone, Copy)]
pub struct WorldGenerator {
    perlin: Perlin,
}

impl Default for WorldGenerator {
    fn default() -> Self {
        Self {
            perlin: Perlin::new(1337),
        }
    }
}

impl WorldGenerator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn generate_chunk(&self, chunk_pos: IVec3) -> ChunkData {
        let mut data = ChunkData::new();

        for x in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                for z in 0..CHUNK_SIZE {
                    let local_pos = USizeVec3::new(x, y, z);

                    let global_pos =
                        Vec3::new(chunk_pos.x as f32, chunk_pos.y as f32, chunk_pos.z as f32)
                            * CHUNK_SIZE as f32
                            + Vec3::new(x as f32, y as f32, z as f32);

                    if global_pos.y < (-global_pos.x.abs() + -global_pos.z.abs()) / 0.01 {
                        data.set_block(local_pos, Block::Rock);
                    }
                }
            }
        }

        data
    }
}
