use glam::IVec3;
use indexmap::IndexMap;
use noise::Perlin;

use crate::{Block, CHUNK_SIZE, Chunk, Vertex, VoxelMeshBuilder};

pub struct Level {
    chunks: IndexMap<IVec3, Chunk>,
}

impl Level {
    pub fn new(
        device: &wgpu::Device,
        chunk_position_bind_group_layout: &wgpu::BindGroupLayout,
    ) -> Self {
        let mut chunks = IndexMap::new();

        let perlin = Perlin::new(1337);

        for x in -8..8 {
            for y in -8..8 {
                for z in -8..8 {
                    let chunk_pos = IVec3::new(x, y, z);
                    let chunk =
                        Chunk::new(device, chunk_position_bind_group_layout, chunk_pos, &perlin);
                    chunks.insert(chunk_pos, chunk);
                }
            }
        }

        Self { chunks }
    }

    pub fn get_block(&self, position: IVec3) -> Option<Block> {
        let chunk_size = CHUNK_SIZE as i32;
        let chunk_pos = position.div_euclid(IVec3::splat(chunk_size));
        let chunk = self.chunks.get(&chunk_pos)?;
        let block_pos = position.rem_euclid(IVec3::splat(chunk_size));
        Some(chunk.get_block(
            block_pos.x as usize,
            block_pos.y as usize,
            block_pos.z as usize,
        ))
    }

    fn is_solid(&self, position: IVec3) -> bool {
        self.get_block(position)
            .is_some_and(|block| block != Block::Air)
    }

    fn calculate_ao(side1: bool, side2: bool, corner: bool) -> u32 {
        if side1 && side2 {
            return 1;
        }
        let count = (side1 as u32) + (side2 as u32) + (corner as u32);
        3 - count
    }

    pub fn update(&mut self, device: &wgpu::Device) {
        let mut meshes = Vec::new();

        for chunk in self.chunks.values() {
            if chunk.is_dirty() {
                let start_pos = chunk.position() * CHUNK_SIZE as i32;

                let mut mesh = VoxelMeshBuilder::new();

                for x in 0..CHUNK_SIZE {
                    for y in 0..CHUNK_SIZE {
                        for z in 0..CHUNK_SIZE {
                            let block = chunk.get_block(x, y, z);

                            if block == Block::Air {
                                continue;
                            }

                            let global_pos = start_pos + IVec3::new(x as i32, y as i32, z as i32);

                            let left = self
                                .get_block(global_pos - IVec3::X)
                                .is_none_or(|block| block == Block::Air);
                            let right = self
                                .get_block(global_pos + IVec3::X)
                                .is_none_or(|block| block == Block::Air);
                            let front = self
                                .get_block(global_pos + IVec3::Z)
                                .is_none_or(|block| block == Block::Air);
                            let back = self
                                .get_block(global_pos - IVec3::Z)
                                .is_none_or(|block| block == Block::Air);
                            let top = self
                                .get_block(global_pos + IVec3::Y)
                                .is_none_or(|block| block == Block::Air);
                            let bottom = self
                                .get_block(global_pos - IVec3::Y)
                                .is_none_or(|block| block == Block::Air);

                            let x = x as u32;
                            let y = y as u32;
                            let z = z as u32;

                            // Front face (+Z)
                            if front {
                                let index = mesh.index();

                                // Calculate AO for each vertex
                                // Top-right vertex
                                let tr_ao = Self::calculate_ao(
                                    self.is_solid(global_pos + IVec3::new(1, 0, 1)),
                                    self.is_solid(global_pos + IVec3::new(0, 1, 1)),
                                    self.is_solid(global_pos + IVec3::new(1, 1, 1)),
                                );
                                // Top-left vertex
                                let tl_ao = Self::calculate_ao(
                                    self.is_solid(global_pos + IVec3::new(-1, 0, 1)),
                                    self.is_solid(global_pos + IVec3::new(0, 1, 1)),
                                    self.is_solid(global_pos + IVec3::new(-1, 1, 1)),
                                );
                                // Bottom-left vertex
                                let bl_ao = Self::calculate_ao(
                                    self.is_solid(global_pos + IVec3::new(-1, 0, 1)),
                                    self.is_solid(global_pos + IVec3::new(0, -1, 1)),
                                    self.is_solid(global_pos + IVec3::new(-1, -1, 1)),
                                );
                                // Bottom-right vertex
                                let br_ao = Self::calculate_ao(
                                    self.is_solid(global_pos + IVec3::new(1, 0, 1)),
                                    self.is_solid(global_pos + IVec3::new(0, -1, 1)),
                                    self.is_solid(global_pos + IVec3::new(1, -1, 1)),
                                );

                                mesh.vertices.push(Vertex::new(
                                    [x + 1, y + 1, z + 1],
                                    [1, 0],
                                    [0, 0, 1],
                                    tr_ao,
                                ));
                                mesh.vertices.push(Vertex::new(
                                    [x, y + 1, z + 1],
                                    [0, 0],
                                    [0, 0, 1],
                                    tl_ao,
                                ));
                                mesh.vertices.push(Vertex::new(
                                    [x, y, z + 1],
                                    [0, 1],
                                    [0, 0, 1],
                                    bl_ao,
                                ));
                                mesh.vertices.push(Vertex::new(
                                    [x + 1, y, z + 1],
                                    [1, 1],
                                    [0, 0, 1],
                                    br_ao,
                                ));

                                mesh.indices.extend_from_slice(&[
                                    index,
                                    index + 1,
                                    index + 2,
                                    index + 2,
                                    index + 3,
                                    index,
                                ]);
                            }

                            // Back face (-Z)
                            if back {
                                let index = mesh.index();

                                // Calculate AO for each vertex
                                // Top-left vertex
                                let tl_ao = Self::calculate_ao(
                                    self.is_solid(global_pos + IVec3::new(-1, 0, -1)),
                                    self.is_solid(global_pos + IVec3::new(0, 1, -1)),
                                    self.is_solid(global_pos + IVec3::new(-1, 1, -1)),
                                );
                                // Top-right vertex
                                let tr_ao = Self::calculate_ao(
                                    self.is_solid(global_pos + IVec3::new(1, 0, -1)),
                                    self.is_solid(global_pos + IVec3::new(0, 1, -1)),
                                    self.is_solid(global_pos + IVec3::new(1, 1, -1)),
                                );
                                // Bottom-right vertex
                                let br_ao = Self::calculate_ao(
                                    self.is_solid(global_pos + IVec3::new(1, 0, -1)),
                                    self.is_solid(global_pos + IVec3::new(0, -1, -1)),
                                    self.is_solid(global_pos + IVec3::new(1, -1, -1)),
                                );
                                // Bottom-left vertex
                                let bl_ao = Self::calculate_ao(
                                    self.is_solid(global_pos + IVec3::new(-1, 0, -1)),
                                    self.is_solid(global_pos + IVec3::new(0, -1, -1)),
                                    self.is_solid(global_pos + IVec3::new(-1, -1, -1)),
                                );

                                mesh.vertices.push(Vertex::new(
                                    [x, y + 1, z],
                                    [1, 0],
                                    [0, 0, -1],
                                    tl_ao,
                                ));
                                mesh.vertices.push(Vertex::new(
                                    [x + 1, y + 1, z],
                                    [0, 0],
                                    [0, 0, -1],
                                    tr_ao,
                                ));
                                mesh.vertices.push(Vertex::new(
                                    [x + 1, y, z],
                                    [0, 1],
                                    [0, 0, -1],
                                    br_ao,
                                ));
                                mesh.vertices.push(Vertex::new(
                                    [x, y, z],
                                    [1, 1],
                                    [0, 0, -1],
                                    bl_ao,
                                ));

                                mesh.indices.extend_from_slice(&[
                                    index,
                                    index + 1,
                                    index + 2,
                                    index + 2,
                                    index + 3,
                                    index,
                                ]);
                            }

                            // Left face (-X)
                            if left {
                                let index = mesh.index();

                                // Calculate AO for each vertex
                                // Top-front vertex
                                let tf_ao = Self::calculate_ao(
                                    self.is_solid(global_pos + IVec3::new(-1, 0, 1)),
                                    self.is_solid(global_pos + IVec3::new(-1, 1, 0)),
                                    self.is_solid(global_pos + IVec3::new(-1, 1, 1)),
                                );
                                // Top-back vertex
                                let tb_ao = Self::calculate_ao(
                                    self.is_solid(global_pos + IVec3::new(-1, 0, -1)),
                                    self.is_solid(global_pos + IVec3::new(-1, 1, 0)),
                                    self.is_solid(global_pos + IVec3::new(-1, 1, -1)),
                                );
                                // Bottom-back vertex
                                let bb_ao = Self::calculate_ao(
                                    self.is_solid(global_pos + IVec3::new(-1, 0, -1)),
                                    self.is_solid(global_pos + IVec3::new(-1, -1, 0)),
                                    self.is_solid(global_pos + IVec3::new(-1, -1, -1)),
                                );
                                // Bottom-front vertex
                                let bf_ao = Self::calculate_ao(
                                    self.is_solid(global_pos + IVec3::new(-1, 0, 1)),
                                    self.is_solid(global_pos + IVec3::new(-1, -1, 0)),
                                    self.is_solid(global_pos + IVec3::new(-1, -1, 1)),
                                );

                                mesh.vertices.push(Vertex::new(
                                    [x, y + 1, z + 1],
                                    [1, 0],
                                    [-1, 0, 0],
                                    tf_ao,
                                ));
                                mesh.vertices.push(Vertex::new(
                                    [x, y + 1, z],
                                    [0, 0],
                                    [-1, 0, 0],
                                    tb_ao,
                                ));
                                mesh.vertices.push(Vertex::new(
                                    [x, y, z],
                                    [0, 1],
                                    [-1, 0, 0],
                                    bb_ao,
                                ));
                                mesh.vertices.push(Vertex::new(
                                    [x, y, z + 1],
                                    [1, 1],
                                    [-1, 0, 0],
                                    bf_ao,
                                ));

                                mesh.indices.extend_from_slice(&[
                                    index,
                                    index + 1,
                                    index + 2,
                                    index + 2,
                                    index + 3,
                                    index,
                                ]);
                            }

                            // Right face (+X)
                            if right {
                                let index = mesh.index();

                                // Calculate AO for each vertex
                                // Top-back vertex
                                let tb_ao = Self::calculate_ao(
                                    self.is_solid(global_pos + IVec3::new(1, 0, -1)),
                                    self.is_solid(global_pos + IVec3::new(1, 1, 0)),
                                    self.is_solid(global_pos + IVec3::new(1, 1, -1)),
                                );
                                // Top-front vertex
                                let tf_ao = Self::calculate_ao(
                                    self.is_solid(global_pos + IVec3::new(1, 0, 1)),
                                    self.is_solid(global_pos + IVec3::new(1, 1, 0)),
                                    self.is_solid(global_pos + IVec3::new(1, 1, 1)),
                                );
                                // Bottom-front vertex
                                let bf_ao = Self::calculate_ao(
                                    self.is_solid(global_pos + IVec3::new(1, 0, 1)),
                                    self.is_solid(global_pos + IVec3::new(1, -1, 0)),
                                    self.is_solid(global_pos + IVec3::new(1, -1, 1)),
                                );
                                // Bottom-back vertex
                                let bb_ao = Self::calculate_ao(
                                    self.is_solid(global_pos + IVec3::new(1, 0, -1)),
                                    self.is_solid(global_pos + IVec3::new(1, -1, 0)),
                                    self.is_solid(global_pos + IVec3::new(1, -1, -1)),
                                );

                                mesh.vertices.push(Vertex::new(
                                    [x + 1, y + 1, z],
                                    [1, 0],
                                    [1, 0, 0],
                                    tb_ao,
                                ));
                                mesh.vertices.push(Vertex::new(
                                    [x + 1, y + 1, z + 1],
                                    [0, 0],
                                    [1, 0, 0],
                                    tf_ao,
                                ));
                                mesh.vertices.push(Vertex::new(
                                    [x + 1, y, z + 1],
                                    [0, 1],
                                    [1, 0, 0],
                                    bf_ao,
                                ));
                                mesh.vertices.push(Vertex::new(
                                    [x + 1, y, z],
                                    [1, 1],
                                    [1, 0, 0],
                                    bb_ao,
                                ));

                                mesh.indices.extend_from_slice(&[
                                    index,
                                    index + 1,
                                    index + 2,
                                    index + 2,
                                    index + 3,
                                    index,
                                ]);
                            }

                            // Top face (+Y)
                            if top {
                                let index = mesh.index();

                                // Calculate AO for each vertex
                                // Front-right vertex
                                let fr_ao = Self::calculate_ao(
                                    self.is_solid(global_pos + IVec3::new(1, 1, 0)),
                                    self.is_solid(global_pos + IVec3::new(0, 1, 1)),
                                    self.is_solid(global_pos + IVec3::new(1, 1, 1)),
                                );
                                // Back-right vertex
                                let br_ao = Self::calculate_ao(
                                    self.is_solid(global_pos + IVec3::new(1, 1, 0)),
                                    self.is_solid(global_pos + IVec3::new(0, 1, -1)),
                                    self.is_solid(global_pos + IVec3::new(1, 1, -1)),
                                );
                                // Back-left vertex
                                let bl_ao = Self::calculate_ao(
                                    self.is_solid(global_pos + IVec3::new(-1, 1, 0)),
                                    self.is_solid(global_pos + IVec3::new(0, 1, -1)),
                                    self.is_solid(global_pos + IVec3::new(-1, 1, -1)),
                                );
                                // Front-left vertex
                                let fl_ao = Self::calculate_ao(
                                    self.is_solid(global_pos + IVec3::new(-1, 1, 0)),
                                    self.is_solid(global_pos + IVec3::new(0, 1, 1)),
                                    self.is_solid(global_pos + IVec3::new(-1, 1, 1)),
                                );

                                mesh.vertices.push(Vertex::new(
                                    [x + 1, y + 1, z + 1],
                                    [1, 1],
                                    [0, 1, 0],
                                    fr_ao,
                                ));
                                mesh.vertices.push(Vertex::new(
                                    [x + 1, y + 1, z],
                                    [1, 0],
                                    [0, 1, 0],
                                    br_ao,
                                ));
                                mesh.vertices.push(Vertex::new(
                                    [x, y + 1, z],
                                    [0, 0],
                                    [0, 1, 0],
                                    bl_ao,
                                ));
                                mesh.vertices.push(Vertex::new(
                                    [x, y + 1, z + 1],
                                    [0, 1],
                                    [0, 1, 0],
                                    fl_ao,
                                ));

                                mesh.indices.extend_from_slice(&[
                                    index,
                                    index + 1,
                                    index + 2,
                                    index + 2,
                                    index + 3,
                                    index,
                                ]);
                            }

                            // Bottom face (-Y)
                            if bottom {
                                let index = mesh.index();

                                // Calculate AO for each vertex
                                // Back-right vertex
                                let br_ao = Self::calculate_ao(
                                    self.is_solid(global_pos + IVec3::new(1, -1, 0)),
                                    self.is_solid(global_pos + IVec3::new(0, -1, -1)),
                                    self.is_solid(global_pos + IVec3::new(1, -1, -1)),
                                );
                                // Front-right vertex
                                let fr_ao = Self::calculate_ao(
                                    self.is_solid(global_pos + IVec3::new(1, -1, 0)),
                                    self.is_solid(global_pos + IVec3::new(0, -1, 1)),
                                    self.is_solid(global_pos + IVec3::new(1, -1, 1)),
                                );
                                // Front-left vertex
                                let fl_ao = Self::calculate_ao(
                                    self.is_solid(global_pos + IVec3::new(-1, -1, 0)),
                                    self.is_solid(global_pos + IVec3::new(0, -1, 1)),
                                    self.is_solid(global_pos + IVec3::new(-1, -1, 1)),
                                );
                                // Back-left vertex
                                let bl_ao = Self::calculate_ao(
                                    self.is_solid(global_pos + IVec3::new(-1, -1, 0)),
                                    self.is_solid(global_pos + IVec3::new(0, -1, -1)),
                                    self.is_solid(global_pos + IVec3::new(-1, -1, -1)),
                                );

                                mesh.vertices.push(Vertex::new(
                                    [x + 1, y, z],
                                    [1, 1],
                                    [0, -1, 0],
                                    br_ao,
                                ));
                                mesh.vertices.push(Vertex::new(
                                    [x + 1, y, z + 1],
                                    [1, 0],
                                    [0, -1, 0],
                                    fr_ao,
                                ));
                                mesh.vertices.push(Vertex::new(
                                    [x, y, z + 1],
                                    [0, 0],
                                    [0, -1, 0],
                                    fl_ao,
                                ));
                                mesh.vertices.push(Vertex::new(
                                    [x, y, z],
                                    [0, 1],
                                    [0, -1, 0],
                                    bl_ao,
                                ));

                                mesh.indices.extend_from_slice(&[
                                    index,
                                    index + 1,
                                    index + 2,
                                    index + 2,
                                    index + 3,
                                    index,
                                ]);
                            }
                        }
                    }
                }

                meshes.push(mesh.build(device));
            }
        }

        for chunk in self.chunks.values_mut() {
            if chunk.is_dirty() {
                chunk.set_mesh(meshes.remove(0));
            }
        }
    }

    pub fn render(&self, render_pass: &mut wgpu::RenderPass) {
        for chunk in self.chunks.values() {
            chunk.render(render_pass);
        }
    }
}
