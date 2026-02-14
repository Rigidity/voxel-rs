use std::collections::VecDeque;
use std::sync::Arc;

use bevy::math::USizeVec3;
use bevy::prelude::*;

use crate::{
    Block, CHUNK_SIZE, LightData, LightDataInner, Registry, RelevantChunks, World,
};

/// Light data for a chunk and its neighbors, used during mesh generation.
#[derive(Debug, Clone)]
pub struct RelevantLights {
    center: LightData,
    center_pos: IVec3,
    neighbors: [(IVec3, LightData); 26],
    neighbor_count: usize,
}

impl RelevantLights {
    pub fn new(center_pos: IVec3, center: LightData) -> Self {
        Self {
            center,
            center_pos,
            neighbors: std::array::from_fn(|_| (IVec3::ZERO, Arc::new(LightDataInner::new()))),
            neighbor_count: 0,
        }
    }

    pub fn add_neighbor(&mut self, pos: IVec3, light: LightData) {
        if self.neighbor_count < 26 {
            self.neighbors[self.neighbor_count] = (pos, light);
            self.neighbor_count += 1;
        }
    }

    pub fn get_light(&self, world_pos: IVec3) -> (u8, u8) {
        let chunk_pos = World::chunk_pos(world_pos);
        let local_pos = World::local_pos(world_pos);

        if chunk_pos == self.center_pos {
            return (
                self.center.get_sky_light(local_pos),
                self.center.get_block_light(local_pos),
            );
        }

        for i in 0..self.neighbor_count {
            if self.neighbors[i].0 == chunk_pos {
                return (
                    self.neighbors[i].1.get_sky_light(local_pos),
                    self.neighbors[i].1.get_block_light(local_pos),
                );
            }
        }

        // No neighbor light data available — default to dark
        (0, 0)
    }
}

/// Check if a position is inside the chunk local coordinate range.
fn is_inside_chunk(local: IVec3) -> bool {
    let cs = CHUNK_SIZE as i32;
    local.x >= 0 && local.x < cs && local.y >= 0 && local.y < cs && local.z >= 0 && local.z < cs
}

/// Propagate skylight for a single chunk. Scans a 1-block border around the
/// chunk so that skylight from neighboring chunks can flow in via BFS.
pub fn propagate_skylight(
    chunk_pos: IVec3,
    chunks: &RelevantChunks,
    registry: &Registry,
) -> LightData {
    let mut light = LightDataInner::new();
    let mut queue: VecDeque<(IVec3, u8)> = VecDeque::new();
    let cs = CHUNK_SIZE as i32;
    let base = chunk_pos * cs;
    let above_base_y = (chunk_pos.y + 1) * cs;

    // Column pass over a wider area: the chunk interior plus a 1-block border
    // in X and Z. Border columns seed the BFS so light can flow across chunk
    // boundaries. We only store light values for interior positions.
    for x in -1..=cs {
        for z in -1..=cs {
            let wx = base.x + x;
            let wz = base.z + z;
            let inside_xz = x >= 0 && x < cs && z >= 0 && z < cs;

            // Determine sky access by scanning the column in the chunk above.
            let mut sky_blocked = false;
            for check_y in (0..CHUNK_SIZE).rev() {
                let above_pos = IVec3::new(wx, above_base_y + check_y as i32, wz);
                if let Some(block) = chunks.get_block(above_pos) {
                    if is_opaque(block, registry) {
                        sky_blocked = true;
                        break;
                    }
                }
            }

            let mut level: u8 = if sky_blocked { 0 } else { 15 };

            for y in (0..CHUNK_SIZE).rev() {
                let world_pos = IVec3::new(wx, base.y + y as i32, wz);

                if let Some(block) = chunks.get_block(world_pos) {
                    if is_opaque(block, registry) {
                        level = 0;
                    }
                }

                if level > 0 {
                    if inside_xz {
                        let local_pos = USizeVec3::new(x as usize, y as usize, z as usize);
                        light.set_sky_light(local_pos, level);
                    }
                    queue.push_back((world_pos, level));
                }
            }
        }
    }

    // BFS flood fill: positions outside the chunk may be in the queue (from
    // the border columns). When popped, they propagate to neighbors that are
    // inside the chunk, allowing light to cross chunk boundaries. We only
    // write light values for positions inside the chunk.
    bfs_propagate_sky(&mut light, &mut queue, chunk_pos, chunks, registry);

    Arc::new(light)
}

fn bfs_propagate_sky(
    light: &mut LightDataInner,
    queue: &mut VecDeque<(IVec3, u8)>,
    chunk_pos: IVec3,
    chunks: &RelevantChunks,
    registry: &Registry,
) {
    let base = chunk_pos * CHUNK_SIZE as i32;

    let directions = [
        IVec3::X,
        IVec3::NEG_X,
        IVec3::Y,
        IVec3::NEG_Y,
        IVec3::Z,
        IVec3::NEG_Z,
    ];

    while let Some((pos, level)) = queue.pop_front() {
        if level <= 1 {
            continue;
        }

        for dir in &directions {
            let neighbor_pos = pos + *dir;
            let local = neighbor_pos - base;

            // Only write to positions inside the chunk
            if !is_inside_chunk(local) {
                continue;
            }

            // Don't propagate through opaque blocks
            if let Some(block) = chunks.get_block(neighbor_pos) {
                if is_opaque(block, registry) {
                    continue;
                }
            }

            let new_level = level - 1;
            let local_pos = USizeVec3::new(local.x as usize, local.y as usize, local.z as usize);

            if light.get_sky_light(local_pos) < new_level {
                light.set_sky_light(local_pos, new_level);
                queue.push_back((neighbor_pos, new_level));
            }
        }
    }
}

/// Propagate block light from emitting blocks within a chunk.
/// Seeds from:
/// 1. Light-emitting blocks inside the chunk
/// 2. Already-computed block light values at the 6 border faces of neighbor chunks
pub fn propagate_block_light(
    light: &mut LightDataInner,
    chunk_pos: IVec3,
    chunks: &RelevantChunks,
    neighbor_lights: &RelevantLights,
    registry: &Registry,
) {
    let cs = CHUNK_SIZE as i32;
    let base = chunk_pos * cs;
    let mut queue: VecDeque<(IVec3, u8)> = VecDeque::new();

    // Seed from light-emitting blocks inside the chunk
    for x in 0..CHUNK_SIZE {
        for y in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                let world_pos = base + IVec3::new(x as i32, y as i32, z as i32);
                let local_pos = USizeVec3::new(x, y, z);

                if let Some(block) = chunks.get_block(world_pos) {
                    let emission = registry.block_type(block.id).light_emission(block.data);
                    if emission > 0 {
                        light.set_block_light(local_pos, emission);
                        queue.push_back((world_pos, emission));
                    }
                }
            }
        }
    }

    // Seed from neighbor chunks' already-computed block light at the 6 border faces.
    // This allows light from sources deep in neighbor chunks to propagate across.
    let border_faces: [(IVec3, [std::ops::Range<i32>; 3]); 6] = [
        (IVec3::new(-1, 0, 0), [-1..-0, 0..cs, 0..cs]),
        (IVec3::new(cs, 0, 0), [cs..cs + 1, 0..cs, 0..cs]),
        (IVec3::new(0, -1, 0), [0..cs, -1..0, 0..cs]),
        (IVec3::new(0, cs, 0), [0..cs, cs..cs + 1, 0..cs]),
        (IVec3::new(0, 0, -1), [0..cs, 0..cs, -1..0]),
        (IVec3::new(0, 0, cs), [0..cs, 0..cs, cs..cs + 1]),
    ];

    for (_normal, ranges) in &border_faces {
        for x in ranges[0].clone() {
            for y in ranges[1].clone() {
                for z in ranges[2].clone() {
                    let world_pos = base + IVec3::new(x, y, z);
                    let (_, block_l) = neighbor_lights.get_light(world_pos);
                    if block_l > 1 {
                        // Don't seed through opaque blocks
                        if let Some(block) = chunks.get_block(world_pos) {
                            if is_opaque(block, registry) {
                                continue;
                            }
                        }
                        queue.push_back((world_pos, block_l));
                    }
                }
            }
        }
    }

    // BFS flood fill — only writes to positions inside the chunk
    let directions = [
        IVec3::X,
        IVec3::NEG_X,
        IVec3::Y,
        IVec3::NEG_Y,
        IVec3::Z,
        IVec3::NEG_Z,
    ];

    while let Some((pos, level)) = queue.pop_front() {
        if level <= 1 {
            continue;
        }

        for dir in &directions {
            let neighbor_pos = pos + *dir;
            let local = neighbor_pos - base;

            if !is_inside_chunk(local) {
                continue;
            }

            if let Some(block) = chunks.get_block(neighbor_pos) {
                if is_opaque(block, registry) {
                    continue;
                }
            }

            let new_level = level - 1;
            let local_pos = USizeVec3::new(local.x as usize, local.y as usize, local.z as usize);

            if light.get_block_light(local_pos) < new_level {
                light.set_block_light(local_pos, new_level);
                queue.push_back((neighbor_pos, new_level));
            }
        }
    }
}

fn is_opaque(block: Block, registry: &Registry) -> bool {
    registry
        .block_type(block.id)
        .face_rect(crate::BlockFace::Top, block.data)
        .is_some_and(|r| !r.is_transparent)
}
