use glam::{IVec3, USizeVec3, Vec3};
use indexmap::IndexMap;
use oneshot::TryRecvError;

use crate::{Block, CHUNK_SIZE, Chunk, ChunkData, WorldGenerator};

#[derive(Debug)]
pub struct World {
    pub chunks: IndexMap<IVec3, Chunk>,
    pub generator: WorldGenerator,
    pub center_pos: IVec3,
    pub generation_tasks: IndexMap<IVec3, oneshot::Receiver<ChunkData>>,
    pub generation_radius: i32,
}

impl World {
    pub fn new(generator: WorldGenerator) -> Self {
        Self {
            chunks: IndexMap::new(),
            generator,
            center_pos: IVec3::ZERO,
            generation_tasks: IndexMap::new(),
            generation_radius: 8,
        }
    }

    pub fn tick(&mut self, center_pos: IVec3) {
        self.center_pos = center_pos;

        let chunks_to_remove = self
            .chunks
            .keys()
            .copied()
            .filter(|chunk_pos| !self.is_visible_chunk(*chunk_pos))
            .collect::<Vec<_>>();

        for chunk_pos in chunks_to_remove {
            self.chunks.swap_remove(&chunk_pos);
        }

        let mut chunks_to_generate = Vec::new();

        for x in -self.generation_radius..=self.generation_radius {
            for y in -self.generation_radius..=self.generation_radius {
                for z in -self.generation_radius..=self.generation_radius {
                    let chunk_pos = center_pos + IVec3::new(x, y, z);

                    if self.is_visible_chunk(chunk_pos)
                        && !self.chunks.contains_key(&chunk_pos)
                        && !self.generation_tasks.contains_key(&chunk_pos)
                    {
                        chunks_to_generate.push(chunk_pos);
                    }
                }
            }
        }

        chunks_to_generate.sort_by_key(|chunk_pos| self.chunk_sort_key(*chunk_pos));

        let mut exceeded = false;

        for chunk_pos in chunks_to_generate {
            if self.generation_tasks.len() >= rayon::current_num_threads() {
                exceeded = true;
                break;
            }

            let (sender, receiver) = oneshot::channel();

            let generator = self.generator.clone();

            rayon::spawn(move || {
                let data = generator.generate_chunk(chunk_pos);
                sender.send(data).ok();
            });

            self.generation_tasks.insert(chunk_pos, receiver);
        }

        let has_generations = !self.generation_tasks.is_empty();

        self.generation_tasks
            .retain(|chunk_pos, receiver| match receiver.try_recv() {
                Ok(data) => {
                    for x in -1..=1 {
                        for y in -1..=1 {
                            for z in -1..=1 {
                                let neighbor = *chunk_pos + IVec3::new(x, y, z);

                                if let Some(neighbor) = self.chunks.get_mut(&neighbor) {
                                    neighbor.queue_remesh();
                                }
                            }
                        }
                    }

                    self.chunks.insert(*chunk_pos, Chunk::new(data));

                    false
                }
                Err(TryRecvError::Disconnected) => false,
                Err(TryRecvError::Empty) => true,
            });

        if has_generations && self.generation_tasks.is_empty() && !exceeded {
            println!(
                "Finished generating chunks, currently have {} loaded chunks",
                self.chunks.len()
            );
        }
    }

    pub fn is_visible_chunk(&self, chunk_pos: IVec3) -> bool {
        let horizontal_distance = Vec3::new(
            chunk_pos.x as f32,
            self.center_pos.y as f32,
            chunk_pos.z as f32,
        )
        .distance(Vec3::new(
            self.center_pos.x as f32,
            self.center_pos.y as f32,
            self.center_pos.z as f32,
        ));

        if horizontal_distance > self.generation_radius as f32 {
            return false;
        }

        let vertical_distance = (chunk_pos.y as f32 - self.center_pos.y as f32).abs();

        if vertical_distance > self.generation_radius as f32 {
            return false;
        }

        true
    }

    pub fn get_block(&self, world_pos: IVec3) -> Option<Block> {
        let chunk_pos = Self::chunk_pos(world_pos);
        let local_pos = Self::local_pos(world_pos);
        let chunk = self.chunks.get(&chunk_pos)?;
        chunk.get_block(local_pos)
    }

    pub fn set_block(&mut self, world_pos: IVec3, block: Option<Block>) {
        let chunk_pos = Self::chunk_pos(world_pos);
        let local_pos = Self::local_pos(world_pos);

        let Some(chunk) = self.chunks.get_mut(&chunk_pos) else {
            return;
        };

        chunk.set_block(local_pos, block);
        chunk.queue_remesh();

        for x in -1..=1 {
            for y in -1..=1 {
                for z in -1..=1 {
                    let neighbor = chunk_pos + IVec3::new(x, y, z);

                    if neighbor == chunk_pos {
                        continue;
                    }

                    if let Some(neighbor) = self.chunks.get_mut(&neighbor) {
                        neighbor.queue_urgent_remesh();
                    }
                }
            }
        }
    }

    pub fn chunk_sort_key(&self, chunk_pos: IVec3) -> (i32, i32) {
        (
            IVec3::new(chunk_pos.x, self.center_pos.y, chunk_pos.z)
                .distance_squared(self.center_pos),
            (chunk_pos.y - self.center_pos.y).abs(),
        )
    }

    pub fn chunk_pos(world_pos: IVec3) -> IVec3 {
        world_pos.div_euclid(IVec3::splat(CHUNK_SIZE as i32))
    }

    pub fn local_pos(world_pos: IVec3) -> USizeVec3 {
        let local_pos = world_pos.rem_euclid(IVec3::splat(CHUNK_SIZE as i32));
        USizeVec3::new(
            local_pos.x as usize,
            local_pos.y as usize,
            local_pos.z as usize,
        )
    }
}
