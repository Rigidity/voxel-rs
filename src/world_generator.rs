use glam::{DVec3, IVec3, USizeVec3};
use noise::{NoiseFn, Perlin};

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

                    let global_pos = DVec3::from(chunk_pos) * CHUNK_SIZE as f64
                        + DVec3::new(x as f64, y as f64, z as f64);

                    let scale1 = 24.0;
                    let scale2 = 12.0;
                    let scale3 = 6.0;

                    let noise1 = self.perlin.get([
                        global_pos.x / scale1,
                        global_pos.y / scale1,
                        global_pos.z / scale1,
                    ]);

                    let noise2 = self.perlin.get([
                        global_pos.x / scale2 + 100.0,
                        global_pos.y / scale2 + 100.0,
                        global_pos.z / scale2 + 100.0,
                    ]) * 0.5;

                    let noise3 = self.perlin.get([
                        global_pos.x / scale3 + 200.0,
                        global_pos.y / scale3 + 200.0,
                        global_pos.z / scale3 + 200.0,
                    ]) * 0.25;

                    let combined_noise = noise1 + noise2 + noise3;

                    if combined_noise > 0.01 {
                        data.set_block(local_pos, Block::Rock);
                    }
                }
            }
        }

        data
    }
}
