use std::sync::Arc;

use bevy::{
    math::{DVec3, USizeVec3},
    prelude::*,
};
use noise::{NoiseFn, Perlin};

use crate::{Block, CHUNK_SIZE, ChunkData, ChunkDataInner, PackedData, Registry};

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

    pub fn generate_chunk(&self, chunk_pos: IVec3, registry: &Registry) -> ChunkData {
        let rock = registry.block_id("rock");
        let soil = registry.block_id("soil");
        let shale = registry.material_id("shale");
        let loam = registry.material_id("loam");
        let lush_grass = registry.material_id("lush_grass");

        let mut data = ChunkDataInner::new();

        for x in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                for z in 0..CHUNK_SIZE {
                    let local_pos = USizeVec3::new(x, y, z);

                    let global_pos =
                        DVec3::new(chunk_pos.x as f64, chunk_pos.y as f64, chunk_pos.z as f64)
                            * CHUNK_SIZE as f64
                            + DVec3::new(x as f64, y as f64, z as f64);

                    let mountain_noise = self
                        .perlin
                        .get([global_pos.x / 400.0, global_pos.z / 400.0])
                        * 40.0;

                    let hill_noise = self
                        .perlin
                        .get([global_pos.x / 150.0, global_pos.z / 150.0])
                        * 20.0;

                    let detail_noise =
                        self.perlin.get([global_pos.x / 50.0, global_pos.z / 50.0]) * 8.0;

                    let fine_noise =
                        self.perlin.get([global_pos.x / 15.0, global_pos.z / 15.0]) * 3.0;

                    let height = 32.0 + mountain_noise + hill_noise + detail_noise + fine_noise;

                    if global_pos.y < height - 3.0 {
                        data.set_block(
                            local_pos,
                            Some(Block::new(
                                rock,
                                PackedData::builder().with_material(shale).build(),
                            )),
                        );
                    } else if global_pos.y < height {
                        let mut block_data = PackedData::builder().with_material(loam);

                        if global_pos.y == height.ceil() - 1.0 {
                            block_data = block_data.with_bool(true).with_material(lush_grass);
                        } else {
                            block_data = block_data.with_bool(false);
                        }

                        data.set_block(local_pos, Some(Block::new(soil, block_data.build())));
                    } else {
                        data.set_block(local_pos, None);
                    }
                }
            }
        }

        Arc::new(data)
    }
}
