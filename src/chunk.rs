use glam::{IVec3, USizeVec3};

use crate::{Block, ChunkData, VoxelMesh, VoxelRenderer, WorldGenerator};

pub const CHUNK_SIZE: usize = 32;

#[derive(Debug)]
pub struct Chunk {
    data: ChunkData,
    bind_group: wgpu::BindGroup,
    mesh: Option<VoxelMesh>,
    is_dirty: bool,
    position: IVec3,
}

impl Chunk {
    pub fn new(device: &wgpu::Device, renderer: &VoxelRenderer, position: IVec3) -> Self {
        let bind_group = renderer.new_chunk_position_bind_group(device, position);

        let world_generator = WorldGenerator::new();

        Self {
            data: world_generator.generate_chunk(position),
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
        self.data.get_block(USizeVec3::new(x, y, z))
    }

    pub fn set_block(&mut self, x: usize, y: usize, z: usize, block: Block) {
        self.data.set_block(USizeVec3::new(x, y, z), block);
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
