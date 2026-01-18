use glam::IVec3;
use wgpu::util::DeviceExt;

use crate::{Block, Vertex, VoxelMesh, VoxelMeshBuilder};

pub const CHUNK_SIZE: usize = 32;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct ChunkData {
    blocks: [Block; CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE],
}

#[derive(Debug)]
pub struct Chunk {
    data: ChunkData,
    bind_group: wgpu::BindGroup,
    mesh: Option<VoxelMesh>,
    is_dirty: bool,
}

impl Chunk {
    pub fn new(
        device: &wgpu::Device,
        chunk_position_bind_group_layout: &wgpu::BindGroupLayout,
        position: IVec3,
    ) -> Self {
        let position_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Chunk Position Buffer"),
            contents: bytemuck::cast_slice(&[ChunkUniform::new(position)]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: chunk_position_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: position_buffer.as_entire_binding(),
            }],
            label: Some("chunk_position_bind_group"),
        });

        Self {
            data: ChunkData {
                blocks: [Block::Rock; CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE],
            },
            bind_group,
            mesh: None,
            is_dirty: true,
        }
    }

    pub fn get_block(&self, x: usize, y: usize, z: usize) -> Block {
        self.data.blocks[x + y * CHUNK_SIZE + z * CHUNK_SIZE * CHUNK_SIZE]
    }

    pub fn set_block(&mut self, x: usize, y: usize, z: usize, block: Block) {
        self.data.blocks[x + y * CHUNK_SIZE + z * CHUNK_SIZE * CHUNK_SIZE] = block;
        self.set_dirty();
    }

    pub fn is_dirty(&self) -> bool {
        self.is_dirty
    }

    pub fn set_dirty(&mut self) {
        self.is_dirty = true;
    }

    pub fn regenerate_mesh(&mut self, device: &wgpu::Device) {
        let mut mesh = VoxelMeshBuilder::new();

        for x in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                for z in 0..CHUNK_SIZE {
                    let block = self.get_block(x, y, z);

                    if block == Block::Air {
                        continue;
                    }

                    let x = x as f32;
                    let y = y as f32;
                    let z = z as f32;

                    // Front face
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

                    // Back face
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

                    // Left face
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

                    // Right face
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

                    // Top face
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

                    // Bottom face
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

        self.mesh = Some(mesh.build(device));
        self.is_dirty = false;
    }

    pub fn render(&self, render_pass: &mut wgpu::RenderPass) {
        if let Some(mesh) = &self.mesh {
            render_pass.set_bind_group(2, &self.bind_group, &[]);
            mesh.draw(render_pass);
        }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct ChunkUniform {
    chunk_position: [f32; 3],
}

impl ChunkUniform {
    fn new(chunk_position: IVec3) -> Self {
        Self {
            chunk_position: [
                chunk_position.x as f32 * CHUNK_SIZE as f32,
                chunk_position.y as f32 * CHUNK_SIZE as f32,
                chunk_position.z as f32 * CHUNK_SIZE as f32,
            ],
        }
    }
}
