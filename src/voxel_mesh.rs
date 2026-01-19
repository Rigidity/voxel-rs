use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;

#[derive(Debug)]
pub struct VoxelMesh {
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    indices: u32,
}

impl VoxelMesh {
    pub fn draw(&self, render_pass: &mut wgpu::RenderPass) {
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        render_pass.draw_indexed(0..self.indices, 0, 0..1);
    }
}

#[derive(Debug, Default)]
pub struct VoxelMeshBuilder {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

impl VoxelMeshBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn index(&self) -> u32 {
        self.vertices.len() as u32
    }

    pub fn build(self, device: &wgpu::Device) -> Option<VoxelMesh> {
        if self.indices.is_empty() {
            return None;
        }

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(self.vertices.as_slice()),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(self.indices.as_slice()),
            usage: wgpu::BufferUsages::INDEX,
        });

        Some(VoxelMesh {
            vertex_buffer,
            index_buffer,
            indices: self.indices.len() as u32,
        })
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct Vertex {
    pub data: u32,
}

impl Vertex {
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
