use bytemuck::{Pod, Zeroable};
use glam::{IVec3, USizeVec3};
use wgpu::util::DeviceExt;

use crate::{Block, CHUNK_SIZE, ChunkData, World};

#[derive(Debug)]
pub struct ChunkMesh {
    position_bind_group: wgpu::BindGroup,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    indices: u32,
}

impl ChunkMesh {
    pub fn draw(&self, render_pass: &mut wgpu::RenderPass) {
        render_pass.set_bind_group(2, &self.position_bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        render_pass.draw_indexed(0..self.indices, 0, 0..1);
    }

    pub fn from_chunk_data(
        device: &wgpu::Device,
        position: IVec3,
        data: &ChunkData,
        world: &World,
        position_bind_group: wgpu::BindGroup,
    ) -> Option<Self> {
        let start_pos = position * CHUNK_SIZE as i32;

        let mut mesh = ChunkMeshBuilder::new();

        for x in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                for z in 0..CHUNK_SIZE {
                    let local_pos = USizeVec3::new(x, y, z);
                    let block = data.get_block(local_pos);

                    if block == Block::Air {
                        continue;
                    }

                    let world_pos = start_pos + IVec3::new(x as i32, y as i32, z as i32);

                    let left = world
                        .get_block(world_pos - IVec3::X)
                        .is_none_or(|block| !block.is_solid());
                    let right = world
                        .get_block(world_pos + IVec3::X)
                        .is_none_or(|block| !block.is_solid());
                    let front = world
                        .get_block(world_pos + IVec3::Z)
                        .is_none_or(|block| !block.is_solid());
                    let back = world
                        .get_block(world_pos - IVec3::Z)
                        .is_none_or(|block| !block.is_solid());
                    let top = world
                        .get_block(world_pos + IVec3::Y)
                        .is_none_or(|block| !block.is_solid());
                    let bottom = world
                        .get_block(world_pos - IVec3::Y)
                        .is_none_or(|block| !block.is_solid());

                    let x = x as u32;
                    let y = y as u32;
                    let z = z as u32;

                    // Front face (+Z)
                    if front {
                        let index = mesh.index();

                        // Calculate AO for each vertex
                        // Top-right vertex
                        let tr_ao = calculate_ao(
                            world
                                .get_block(world_pos + IVec3::new(1, 0, 1))
                                .is_some_and(|block| block.is_solid()),
                            world
                                .get_block(world_pos + IVec3::new(0, 1, 1))
                                .is_some_and(|block| block.is_solid()),
                            world
                                .get_block(world_pos + IVec3::new(1, 1, 1))
                                .is_some_and(|block| block.is_solid()),
                        );
                        // Top-left vertex
                        let tl_ao = calculate_ao(
                            world
                                .get_block(world_pos + IVec3::new(-1, 0, 1))
                                .is_some_and(|block| block.is_solid()),
                            world
                                .get_block(world_pos + IVec3::new(0, 1, 1))
                                .is_some_and(|block| block.is_solid()),
                            world
                                .get_block(world_pos + IVec3::new(-1, 1, 1))
                                .is_some_and(|block| block.is_solid()),
                        );
                        // Bottom-left vertex
                        let bl_ao = calculate_ao(
                            world
                                .get_block(world_pos + IVec3::new(-1, 0, 1))
                                .is_some_and(|block| block.is_solid()),
                            world
                                .get_block(world_pos + IVec3::new(0, -1, 1))
                                .is_some_and(|block| block.is_solid()),
                            world
                                .get_block(world_pos + IVec3::new(-1, -1, 1))
                                .is_some_and(|block| block.is_solid()),
                        );
                        // Bottom-right vertex
                        let br_ao = calculate_ao(
                            world
                                .get_block(world_pos + IVec3::new(1, 0, 1))
                                .is_some_and(|block| block.is_solid()),
                            world
                                .get_block(world_pos + IVec3::new(0, -1, 1))
                                .is_some_and(|block| block.is_solid()),
                            world
                                .get_block(world_pos + IVec3::new(1, -1, 1))
                                .is_some_and(|block| block.is_solid()),
                        );

                        mesh.vertices.push(ChunkVertex::new(
                            [x + 1, y + 1, z + 1],
                            [1, 0],
                            [0, 0, 1],
                            tr_ao,
                        ));
                        mesh.vertices.push(ChunkVertex::new(
                            [x, y + 1, z + 1],
                            [0, 0],
                            [0, 0, 1],
                            tl_ao,
                        ));
                        mesh.vertices.push(ChunkVertex::new(
                            [x, y, z + 1],
                            [0, 1],
                            [0, 0, 1],
                            bl_ao,
                        ));
                        mesh.vertices.push(ChunkVertex::new(
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
                        let tl_ao = calculate_ao(
                            world
                                .get_block(world_pos + IVec3::new(-1, 0, -1))
                                .is_some_and(|block| block.is_solid()),
                            world
                                .get_block(world_pos + IVec3::new(0, 1, -1))
                                .is_some_and(|block| block.is_solid()),
                            world
                                .get_block(world_pos + IVec3::new(-1, 1, -1))
                                .is_some_and(|block| block.is_solid()),
                        );
                        // Top-right vertex
                        let tr_ao = calculate_ao(
                            world
                                .get_block(world_pos + IVec3::new(1, 0, -1))
                                .is_some_and(|block| block.is_solid()),
                            world
                                .get_block(world_pos + IVec3::new(0, 1, -1))
                                .is_some_and(|block| block.is_solid()),
                            world
                                .get_block(world_pos + IVec3::new(1, 1, -1))
                                .is_some_and(|block| block.is_solid()),
                        );
                        // Bottom-right vertex
                        let br_ao = calculate_ao(
                            world
                                .get_block(world_pos + IVec3::new(1, 0, -1))
                                .is_some_and(|block| block.is_solid()),
                            world
                                .get_block(world_pos + IVec3::new(0, -1, -1))
                                .is_some_and(|block| block.is_solid()),
                            world
                                .get_block(world_pos + IVec3::new(1, -1, -1))
                                .is_some_and(|block| block.is_solid()),
                        );
                        // Bottom-left vertex
                        let bl_ao = calculate_ao(
                            world
                                .get_block(world_pos + IVec3::new(-1, 0, -1))
                                .is_some_and(|block| block.is_solid()),
                            world
                                .get_block(world_pos + IVec3::new(0, -1, -1))
                                .is_some_and(|block| block.is_solid()),
                            world
                                .get_block(world_pos + IVec3::new(-1, -1, -1))
                                .is_some_and(|block| block.is_solid()),
                        );

                        mesh.vertices.push(ChunkVertex::new(
                            [x, y + 1, z],
                            [1, 0],
                            [0, 0, -1],
                            tl_ao,
                        ));
                        mesh.vertices.push(ChunkVertex::new(
                            [x + 1, y + 1, z],
                            [0, 0],
                            [0, 0, -1],
                            tr_ao,
                        ));
                        mesh.vertices.push(ChunkVertex::new(
                            [x + 1, y, z],
                            [0, 1],
                            [0, 0, -1],
                            br_ao,
                        ));
                        mesh.vertices
                            .push(ChunkVertex::new([x, y, z], [1, 1], [0, 0, -1], bl_ao));

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
                        let tf_ao = calculate_ao(
                            world
                                .get_block(world_pos + IVec3::new(-1, 0, 1))
                                .is_some_and(|block| block.is_solid()),
                            world
                                .get_block(world_pos + IVec3::new(-1, 1, 0))
                                .is_some_and(|block| block.is_solid()),
                            world
                                .get_block(world_pos + IVec3::new(-1, 1, 1))
                                .is_some_and(|block| block.is_solid()),
                        );
                        // Top-back vertex
                        let tb_ao = calculate_ao(
                            world
                                .get_block(world_pos + IVec3::new(-1, 0, -1))
                                .is_some_and(|block| block.is_solid()),
                            world
                                .get_block(world_pos + IVec3::new(-1, 1, 0))
                                .is_some_and(|block| block.is_solid()),
                            world
                                .get_block(world_pos + IVec3::new(-1, 1, -1))
                                .is_some_and(|block| block.is_solid()),
                        );
                        // Bottom-back vertex
                        let bb_ao = calculate_ao(
                            world
                                .get_block(world_pos + IVec3::new(-1, 0, -1))
                                .is_some_and(|block| block.is_solid()),
                            world
                                .get_block(world_pos + IVec3::new(-1, -1, 0))
                                .is_some_and(|block| block.is_solid()),
                            world
                                .get_block(world_pos + IVec3::new(-1, -1, -1))
                                .is_some_and(|block| block.is_solid()),
                        );
                        // Bottom-front vertex
                        let bf_ao = calculate_ao(
                            world
                                .get_block(world_pos + IVec3::new(-1, 0, 1))
                                .is_some_and(|block| block.is_solid()),
                            world
                                .get_block(world_pos + IVec3::new(-1, -1, 0))
                                .is_some_and(|block| block.is_solid()),
                            world
                                .get_block(world_pos + IVec3::new(-1, -1, 1))
                                .is_some_and(|block| block.is_solid()),
                        );

                        mesh.vertices.push(ChunkVertex::new(
                            [x, y + 1, z + 1],
                            [1, 0],
                            [-1, 0, 0],
                            tf_ao,
                        ));
                        mesh.vertices.push(ChunkVertex::new(
                            [x, y + 1, z],
                            [0, 0],
                            [-1, 0, 0],
                            tb_ao,
                        ));
                        mesh.vertices
                            .push(ChunkVertex::new([x, y, z], [0, 1], [-1, 0, 0], bb_ao));
                        mesh.vertices.push(ChunkVertex::new(
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
                        let tb_ao = calculate_ao(
                            world
                                .get_block(world_pos + IVec3::new(1, 0, -1))
                                .is_some_and(|block| block.is_solid()),
                            world
                                .get_block(world_pos + IVec3::new(1, 1, 0))
                                .is_some_and(|block| block.is_solid()),
                            world
                                .get_block(world_pos + IVec3::new(1, 1, -1))
                                .is_some_and(|block| block.is_solid()),
                        );
                        // Top-front vertex
                        let tf_ao = calculate_ao(
                            world
                                .get_block(world_pos + IVec3::new(1, 0, 1))
                                .is_some_and(|block| block.is_solid()),
                            world
                                .get_block(world_pos + IVec3::new(1, 1, 0))
                                .is_some_and(|block| block.is_solid()),
                            world
                                .get_block(world_pos + IVec3::new(1, 1, 1))
                                .is_some_and(|block| block.is_solid()),
                        );
                        // Bottom-front vertex
                        let bf_ao = calculate_ao(
                            world
                                .get_block(world_pos + IVec3::new(1, 0, 1))
                                .is_some_and(|block| block.is_solid()),
                            world
                                .get_block(world_pos + IVec3::new(1, -1, 0))
                                .is_some_and(|block| block.is_solid()),
                            world
                                .get_block(world_pos + IVec3::new(1, -1, 1))
                                .is_some_and(|block| block.is_solid()),
                        );
                        // Bottom-back vertex
                        let bb_ao = calculate_ao(
                            world
                                .get_block(world_pos + IVec3::new(1, 0, -1))
                                .is_some_and(|block| block.is_solid()),
                            world
                                .get_block(world_pos + IVec3::new(1, -1, 0))
                                .is_some_and(|block| block.is_solid()),
                            world
                                .get_block(world_pos + IVec3::new(1, -1, -1))
                                .is_some_and(|block| block.is_solid()),
                        );

                        mesh.vertices.push(ChunkVertex::new(
                            [x + 1, y + 1, z],
                            [1, 0],
                            [1, 0, 0],
                            tb_ao,
                        ));
                        mesh.vertices.push(ChunkVertex::new(
                            [x + 1, y + 1, z + 1],
                            [0, 0],
                            [1, 0, 0],
                            tf_ao,
                        ));
                        mesh.vertices.push(ChunkVertex::new(
                            [x + 1, y, z + 1],
                            [0, 1],
                            [1, 0, 0],
                            bf_ao,
                        ));
                        mesh.vertices.push(ChunkVertex::new(
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
                        let fr_ao = calculate_ao(
                            world
                                .get_block(world_pos + IVec3::new(1, 1, 0))
                                .is_some_and(|block| block.is_solid()),
                            world
                                .get_block(world_pos + IVec3::new(0, 1, 1))
                                .is_some_and(|block| block.is_solid()),
                            world
                                .get_block(world_pos + IVec3::new(1, 1, 1))
                                .is_some_and(|block| block.is_solid()),
                        );
                        // Back-right vertex
                        let br_ao = calculate_ao(
                            world
                                .get_block(world_pos + IVec3::new(1, 1, 0))
                                .is_some_and(|block| block.is_solid()),
                            world
                                .get_block(world_pos + IVec3::new(0, 1, -1))
                                .is_some_and(|block| block.is_solid()),
                            world
                                .get_block(world_pos + IVec3::new(1, 1, -1))
                                .is_some_and(|block| block.is_solid()),
                        );
                        // Back-left vertex
                        let bl_ao = calculate_ao(
                            world
                                .get_block(world_pos + IVec3::new(-1, 1, 0))
                                .is_some_and(|block| block.is_solid()),
                            world
                                .get_block(world_pos + IVec3::new(0, 1, -1))
                                .is_some_and(|block| block.is_solid()),
                            world
                                .get_block(world_pos + IVec3::new(-1, 1, -1))
                                .is_some_and(|block| block.is_solid()),
                        );
                        // Front-left vertex
                        let fl_ao = calculate_ao(
                            world
                                .get_block(world_pos + IVec3::new(-1, 1, 0))
                                .is_some_and(|block| block.is_solid()),
                            world
                                .get_block(world_pos + IVec3::new(0, 1, 1))
                                .is_some_and(|block| block.is_solid()),
                            world
                                .get_block(world_pos + IVec3::new(-1, 1, 1))
                                .is_some_and(|block| block.is_solid()),
                        );

                        mesh.vertices.push(ChunkVertex::new(
                            [x + 1, y + 1, z + 1],
                            [1, 1],
                            [0, 1, 0],
                            fr_ao,
                        ));
                        mesh.vertices.push(ChunkVertex::new(
                            [x + 1, y + 1, z],
                            [1, 0],
                            [0, 1, 0],
                            br_ao,
                        ));
                        mesh.vertices.push(ChunkVertex::new(
                            [x, y + 1, z],
                            [0, 0],
                            [0, 1, 0],
                            bl_ao,
                        ));
                        mesh.vertices.push(ChunkVertex::new(
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
                        let br_ao = calculate_ao(
                            world
                                .get_block(world_pos + IVec3::new(1, -1, 0))
                                .is_some_and(|block| block.is_solid()),
                            world
                                .get_block(world_pos + IVec3::new(0, -1, -1))
                                .is_some_and(|block| block.is_solid()),
                            world
                                .get_block(world_pos + IVec3::new(1, -1, -1))
                                .is_some_and(|block| block.is_solid()),
                        );
                        // Front-right vertex
                        let fr_ao = calculate_ao(
                            world
                                .get_block(world_pos + IVec3::new(1, -1, 0))
                                .is_some_and(|block| block.is_solid()),
                            world
                                .get_block(world_pos + IVec3::new(0, -1, 1))
                                .is_some_and(|block| block.is_solid()),
                            world
                                .get_block(world_pos + IVec3::new(1, -1, 1))
                                .is_some_and(|block| block.is_solid()),
                        );
                        // Front-left vertex
                        let fl_ao = calculate_ao(
                            world
                                .get_block(world_pos + IVec3::new(-1, -1, 0))
                                .is_some_and(|block| block.is_solid()),
                            world
                                .get_block(world_pos + IVec3::new(0, -1, 1))
                                .is_some_and(|block| block.is_solid()),
                            world
                                .get_block(world_pos + IVec3::new(-1, -1, 1))
                                .is_some_and(|block| block.is_solid()),
                        );
                        // Back-left vertex
                        let bl_ao = calculate_ao(
                            world
                                .get_block(world_pos + IVec3::new(-1, -1, 0))
                                .is_some_and(|block| block.is_solid()),
                            world
                                .get_block(world_pos + IVec3::new(0, -1, -1))
                                .is_some_and(|block| block.is_solid()),
                            world
                                .get_block(world_pos + IVec3::new(-1, -1, -1))
                                .is_some_and(|block| block.is_solid()),
                        );

                        mesh.vertices.push(ChunkVertex::new(
                            [x + 1, y, z],
                            [1, 1],
                            [0, -1, 0],
                            br_ao,
                        ));
                        mesh.vertices.push(ChunkVertex::new(
                            [x + 1, y, z + 1],
                            [1, 0],
                            [0, -1, 0],
                            fr_ao,
                        ));
                        mesh.vertices.push(ChunkVertex::new(
                            [x, y, z + 1],
                            [0, 0],
                            [0, -1, 0],
                            fl_ao,
                        ));
                        mesh.vertices
                            .push(ChunkVertex::new([x, y, z], [0, 1], [0, -1, 0], bl_ao));

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

        mesh.build(device, position_bind_group)
    }
}

#[derive(Debug, Default)]
pub struct ChunkMeshBuilder {
    pub vertices: Vec<ChunkVertex>,
    pub indices: Vec<u32>,
}

impl ChunkMeshBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn index(&self) -> u32 {
        self.vertices.len() as u32
    }

    pub fn build(
        self,
        device: &wgpu::Device,
        position_bind_group: wgpu::BindGroup,
    ) -> Option<ChunkMesh> {
        if self.indices.is_empty() {
            return None;
        }

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("ChunkVertex Buffer"),
            contents: bytemuck::cast_slice(self.vertices.as_slice()),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(self.indices.as_slice()),
            usage: wgpu::BufferUsages::INDEX,
        });

        Some(ChunkMesh {
            position_bind_group,
            vertex_buffer,
            index_buffer,
            indices: self.indices.len() as u32,
        })
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct ChunkVertex {
    pub data: u32,
}

impl ChunkVertex {
    pub const ATTRIBS: [wgpu::VertexAttribute; 1] = wgpu::vertex_attr_array![0 => Uint32];

    pub fn new(position: [u32; 3], tex_coords: [u32; 2], _normal: [i32; 3], ao: u32) -> Self {
        Self {
            data: (position[0] << 26)
                | (position[1] << 20)
                | (position[2] << 14)
                | (tex_coords[0] << 13)
                | (tex_coords[1] << 12)
                | (ao << 10),
        }
    }

    pub fn descriptor() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;

        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

fn calculate_ao(side1: bool, side2: bool, corner: bool) -> u32 {
    if side1 && side2 {
        return 1;
    }
    let count = (side1 as u32) + (side2 as u32) + (corner as u32);
    3 - count
}
