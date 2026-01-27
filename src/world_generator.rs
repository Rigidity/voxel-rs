use glam::{DVec3, IVec3, USizeVec3};
use noise::{NoiseFn, Perlin};

use crate::{Block, BlockData, BlockKind, CHUNK_SIZE, ChunkData, Material, RockData, SoilData};

#[derive(Debug, Clone)]
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
                        DVec3::new(chunk_pos.x as f64, chunk_pos.y as f64, chunk_pos.z as f64)
                            * CHUNK_SIZE as f64
                            + DVec3::new(x as f64, y as f64, z as f64);

                    let value = self.sample(
                        DVec3::new(global_pos.x / 2.0, global_pos.y, global_pos.z / 2.0),
                        3,
                        1.0 / CHUNK_SIZE as f64,
                        1.0,
                    );

                    if value > -0.1 {
                        let block = if global_pos.x < 0.0 {
                            let rock_data = RockData {
                                material: if global_pos.y < 0.0 {
                                    Material::Shale
                                } else {
                                    Material::Chalk
                                },
                            };
                            Block::new(BlockKind::Rock, rock_data.encode())
                        } else {
                            let soil_data = SoilData {
                                material: if global_pos.y < -200.0 {
                                    Material::Loam
                                } else if global_pos.y < -100.0 {
                                    Material::Silt
                                } else {
                                    Material::Clay
                                },
                            };
                            Block::new(BlockKind::Soil, soil_data.encode())
                        };

                        data.set_block(local_pos, Some(block));
                    }
                }
            }
        }

        data
    }

    fn sample(&self, global_pos: DVec3, octaves: u32, frequency: f64, amplitude: f64) -> f64 {
        let mut value = 0.0;
        let mut frequency = frequency;
        let mut amplitude = amplitude;

        for _ in 0..octaves {
            value += self.perlin.get([
                global_pos.x * frequency,
                global_pos.y * frequency,
                global_pos.z * frequency,
            ]) * amplitude;
            frequency *= 2.0;
            amplitude *= 0.5;
        }

        value
    }
}
