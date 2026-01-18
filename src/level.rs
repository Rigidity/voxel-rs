use glam::IVec3;
use indexmap::IndexMap;

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

        for x in -4..4 {
            for y in -4..4 {
                for z in -4..4 {
                    let chunk_pos = IVec3::new(x, y, z);
                    let chunk = Chunk::new(device, chunk_position_bind_group_layout, chunk_pos);
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

                            let left = self.get_block(global_pos - IVec3::X).is_none();
                            let right = self.get_block(global_pos + IVec3::X).is_none();
                            let front = self.get_block(global_pos + IVec3::Z).is_none();
                            let back = self.get_block(global_pos - IVec3::Z).is_none();
                            let top = self.get_block(global_pos + IVec3::Y).is_none();
                            let bottom = self.get_block(global_pos - IVec3::Y).is_none();

                            let x = x as f32;
                            let y = y as f32;
                            let z = z as f32;

                            // Front face
                            if front {
                                let index = mesh.index();

                                mesh.vertices
                                    .push(Vertex::new([x + 1.0, y + 1.0, z + 1.0], [1.0, 0.0]));
                                mesh.vertices
                                    .push(Vertex::new([x + 0.0, y + 1.0, z + 1.0], [0.0, 0.0]));
                                mesh.vertices
                                    .push(Vertex::new([x + 0.0, y + 0.0, z + 1.0], [0.0, 1.0]));
                                mesh.vertices
                                    .push(Vertex::new([x + 1.0, y + 0.0, z + 1.0], [1.0, 1.0]));

                                mesh.indices.extend_from_slice(&[
                                    index,
                                    index + 1,
                                    index + 2,
                                    index + 2,
                                    index + 3,
                                    index,
                                ]);
                            }

                            // Back face
                            if back {
                                let index = mesh.index();

                                mesh.vertices
                                    .push(Vertex::new([x + 0.0, y + 1.0, z + 0.0], [1.0, 0.0]));
                                mesh.vertices
                                    .push(Vertex::new([x + 1.0, y + 1.0, z + 0.0], [0.0, 0.0]));
                                mesh.vertices
                                    .push(Vertex::new([x + 1.0, y + 0.0, z + 0.0], [0.0, 1.0]));
                                mesh.vertices
                                    .push(Vertex::new([x + 0.0, y + 0.0, z + 0.0], [1.0, 1.0]));

                                mesh.indices.extend_from_slice(&[
                                    index,
                                    index + 1,
                                    index + 2,
                                    index + 2,
                                    index + 3,
                                    index,
                                ]);
                            }

                            // Left face
                            if left {
                                let index = mesh.index();

                                mesh.vertices
                                    .push(Vertex::new([x + 0.0, y + 1.0, z + 1.0], [1.0, 0.0]));
                                mesh.vertices
                                    .push(Vertex::new([x + 0.0, y + 1.0, z + 0.0], [0.0, 0.0]));
                                mesh.vertices
                                    .push(Vertex::new([x + 0.0, y + 0.0, z + 0.0], [0.0, 1.0]));
                                mesh.vertices
                                    .push(Vertex::new([x + 0.0, y + 0.0, z + 1.0], [1.0, 1.0]));

                                mesh.indices.extend_from_slice(&[
                                    index,
                                    index + 1,
                                    index + 2,
                                    index + 2,
                                    index + 3,
                                    index,
                                ]);
                            }

                            // Right face
                            if right {
                                let index = mesh.index();

                                mesh.vertices
                                    .push(Vertex::new([x + 1.0, y + 1.0, z + 0.0], [1.0, 0.0]));
                                mesh.vertices
                                    .push(Vertex::new([x + 1.0, y + 1.0, z + 1.0], [0.0, 0.0]));
                                mesh.vertices
                                    .push(Vertex::new([x + 1.0, y + 0.0, z + 1.0], [0.0, 1.0]));
                                mesh.vertices
                                    .push(Vertex::new([x + 1.0, y + 0.0, z + 0.0], [1.0, 1.0]));

                                mesh.indices.extend_from_slice(&[
                                    index,
                                    index + 1,
                                    index + 2,
                                    index + 2,
                                    index + 3,
                                    index,
                                ]);
                            }

                            // Top face
                            if top {
                                let index = mesh.index();

                                mesh.vertices
                                    .push(Vertex::new([x + 1.0, y + 1.0, z + 1.0], [1.0, 1.0]));
                                mesh.vertices
                                    .push(Vertex::new([x + 1.0, y + 1.0, z + 0.0], [1.0, 0.0]));
                                mesh.vertices
                                    .push(Vertex::new([x + 0.0, y + 1.0, z + 0.0], [0.0, 0.0]));
                                mesh.vertices
                                    .push(Vertex::new([x + 0.0, y + 1.0, z + 1.0], [0.0, 1.0]));

                                mesh.indices.extend_from_slice(&[
                                    index,
                                    index + 1,
                                    index + 2,
                                    index + 2,
                                    index + 3,
                                    index,
                                ]);
                            }

                            // Bottom face
                            if bottom {
                                let index = mesh.index();

                                mesh.vertices
                                    .push(Vertex::new([x + 1.0, y + 0.0, z + 0.0], [1.0, 1.0]));
                                mesh.vertices
                                    .push(Vertex::new([x + 1.0, y + 0.0, z + 1.0], [1.0, 0.0]));
                                mesh.vertices
                                    .push(Vertex::new([x + 0.0, y + 0.0, z + 1.0], [0.0, 0.0]));
                                mesh.vertices
                                    .push(Vertex::new([x + 0.0, y + 0.0, z + 0.0], [0.0, 1.0]));

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
                break;
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
