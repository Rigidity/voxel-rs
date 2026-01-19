use glam::IVec3;
use noise::{NoiseFn, Perlin};
use wgpu::util::DeviceExt;

use crate::{Block, VoxelMesh};

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
    position: IVec3,
}

impl Chunk {
    pub fn new(
        device: &wgpu::Device,
        chunk_position_bind_group_layout: &wgpu::BindGroupLayout,
        position: IVec3,
        perlin: &Perlin,
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

        let mut block_data = [Block::Air; CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE];

        for x in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                for z in 0..CHUNK_SIZE {
                    let global_pos =
                        position * CHUNK_SIZE as i32 + IVec3::new(x as i32, y as i32, z as i32);

                    // Use multiple octaves of noise for more interesting terrain
                    let scale1 = 24.0;
                    let scale2 = 12.0;
                    let scale3 = 6.0;

                    let noise1 = perlin.get([
                        global_pos.x as f64 / scale1,
                        global_pos.y as f64 / scale1,
                        global_pos.z as f64 / scale1,
                    ]);

                    let noise2 = perlin.get([
                        global_pos.x as f64 / scale2 + 100.0,
                        global_pos.y as f64 / scale2 + 100.0,
                        global_pos.z as f64 / scale2 + 100.0,
                    ]) * 0.5;

                    let noise3 = perlin.get([
                        global_pos.x as f64 / scale3 + 200.0,
                        global_pos.y as f64 / scale3 + 200.0,
                        global_pos.z as f64 / scale3 + 200.0,
                    ]) * 0.25;

                    // Combine the octaves for richer detail
                    let combined_noise = noise1 + noise2 + noise3;

                    // Create interesting cave-like structures
                    if combined_noise > 0.01 {
                        block_data[x + y * CHUNK_SIZE + z * CHUNK_SIZE * CHUNK_SIZE] = Block::Rock;
                    }
                }
            }
        }

        Self {
            data: ChunkData { blocks: block_data },
            bind_group,
            mesh: None,
            is_dirty: true,
            position,
        }
    }

    pub fn position(&self) -> IVec3 {
        self.position
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

    pub fn set_mesh(&mut self, mesh: Option<VoxelMesh>) {
        self.mesh = mesh;
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
