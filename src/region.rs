use std::{
    collections::{HashMap, HashSet},
    fs,
    path::PathBuf,
};

use bevy::math::IVec3;
use indexmap::IndexMap;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};

use crate::{Block, ChunkData};

const REGION_SIZE: usize = 16;

pub struct RegionManager {
    region_dir: PathBuf,
    loaded_regions: RwLock<HashMap<IVec3, Region>>,
}

impl RegionManager {
    pub fn new(region_dir: PathBuf) -> Self {
        if !region_dir.try_exists().unwrap() {
            fs::create_dir_all(&region_dir).unwrap();
        }

        Self {
            region_dir,
            loaded_regions: RwLock::new(HashMap::new()),
        }
    }

    pub fn load_chunk(&self, chunk_pos: IVec3) -> Option<ChunkData> {
        let region_pos = self.region_pos(chunk_pos);

        if let Some(region) = self.loaded_regions.read().get(&region_pos) {
            return Some(region.chunks.get(&chunk_pos)?.decompress());
        }

        let mut loaded_regions = self.loaded_regions.write();

        let path = self.file_path(region_pos);

        if !path.exists() {
            return None;
        }

        let data = fs::read(path).unwrap();
        let region: Region = postcard::from_bytes(&data).unwrap();

        let chunk_data = region
            .chunks
            .get(&chunk_pos)
            .map(CompressedChunk::decompress);

        loaded_regions.insert(region_pos, region);

        chunk_data
    }

    pub fn save_chunks(&self, chunks: &HashMap<IVec3, ChunkData>) {
        let mut loaded_regions = self.loaded_regions.write();
        let mut modified_regions = HashSet::new();

        for (chunk_pos, chunk_data) in chunks {
            let region_pos = self.region_pos(*chunk_pos);
            let region = loaded_regions.entry(region_pos).or_default();
            region
                .chunks
                .insert(*chunk_pos, CompressedChunk::compress(chunk_data));
            modified_regions.insert(region_pos);
        }

        for region_pos in modified_regions {
            let path = self.file_path(region_pos);
            let region = loaded_regions.get(&region_pos).unwrap();
            fs::write(path, postcard::to_allocvec(region).unwrap()).unwrap();
        }
    }

    fn file_path(&self, region_pos: IVec3) -> PathBuf {
        self.region_dir.join(format!(
            "region_{}_{}_{}.bin",
            region_pos.x, region_pos.y, region_pos.z
        ))
    }

    fn region_pos(&self, chunk_pos: IVec3) -> IVec3 {
        chunk_pos.div_euclid(IVec3::splat(REGION_SIZE as i32))
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
struct Region {
    chunks: IndexMap<IVec3, CompressedChunk>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CompressedChunk {
    palette: Vec<Block>,
    rle_data: Vec<u16>,
}

impl CompressedChunk {
    fn compress(data: &ChunkData) -> Self {
        let mut palette_builder = PaletteBuilder::default();
        let mut rle_data = Vec::new();
        let mut current_block = None;
        let mut length = 0;

        for block in data.clone_data() {
            if let Some(current_block) = &mut current_block {
                if *current_block == block {
                    length += 1;
                    continue;
                }

                rle_data.push(length);
                rle_data.push(palette_builder.get_index(*current_block));
                *current_block = block;
                length = 1;
            } else {
                current_block = Some(block);
                length = 1;
            }
        }

        if let Some(current_block) = current_block
            && length > 0
        {
            rle_data.push(length);
            rle_data.push(palette_builder.get_index(current_block));
        }

        Self {
            palette: palette_builder.palette,
            rle_data,
        }
    }

    fn decompress(&self) -> ChunkData {
        let mut data = Vec::new();

        let mut i = 0;

        while i < self.rle_data.len() {
            let palette_index = self.rle_data[i + 1];
            let block = self.get_block(palette_index);

            for _ in 0..self.rle_data[i] {
                data.push(block);
            }

            i += 2;
        }

        ChunkData::from_data(data)
    }

    fn get_block(&self, palette_index: u16) -> Option<Block> {
        if palette_index == 0 {
            return None;
        }

        Some(self.palette[palette_index as usize - 1])
    }
}

#[derive(Default)]
struct PaletteBuilder {
    palette: Vec<Block>,
    palette_map: HashMap<Block, u16>,
}

impl PaletteBuilder {
    fn get_index(&mut self, block: Option<Block>) -> u16 {
        if let Some(block) = block {
            if let Some(index) = self.palette_map.get(&block) {
                *index
            } else {
                self.palette.push(block);
                let index = self.palette.len() as u16;
                self.palette_map.insert(block, index);
                index
            }
        } else {
            0
        }
    }
}
