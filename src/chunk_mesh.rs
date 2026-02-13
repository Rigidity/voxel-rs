use bevy::{
    asset::RenderAssetUsages,
    math::USizeVec3,
    mesh::{Indices, PrimitiveTopology},
    prelude::*,
};

use crate::{
    ATTRIBUTE_PACKED_DATA, ATTRIBUTE_TEXTURE_INDEX, CHUNK_SIZE, ModelId, Registry, RelevantChunks,
    RenderContext,
};

pub fn generate_mesh(
    center_pos: IVec3,
    data: &RelevantChunks,
    registry: &Registry,
) -> Option<Mesh> {
    let mut mesh = ChunkMeshBuilder::new();

    for x in 0..CHUNK_SIZE {
        for y in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                let world_pos =
                    center_pos * CHUNK_SIZE as i32 + IVec3::new(x as i32, y as i32, z as i32);

                let Some(block) = data.get_block(world_pos) else {
                    continue;
                };

                registry.block_type(block.id).render(&mut RenderContext {
                    data,
                    local_pos: USizeVec3::new(x, y, z),
                    world_pos,
                    mesh: &mut mesh,
                    block,
                    registry,
                });
            }
        }
    }

    mesh.build()
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

    pub fn build(self) -> Option<Mesh> {
        if self.indices.is_empty() {
            return None;
        }

        let mut packed_data = Vec::new();
        let mut texture_indices = Vec::new();

        for vertex in self.vertices {
            packed_data.push(vertex.data);
            texture_indices.push(vertex.texture_index);
        }

        let mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD,
        )
        .with_inserted_attribute(ATTRIBUTE_PACKED_DATA, packed_data)
        .with_inserted_attribute(ATTRIBUTE_TEXTURE_INDEX, texture_indices)
        .with_inserted_indices(Indices::U32(self.indices));

        Some(mesh)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ChunkVertex {
    pub data: u32,
    pub texture_index: u32,
}

impl ChunkVertex {
    /// Creates a new chunk vertex with bitpacked data
    /// Packing layout (30 bits total in u32):
    /// - Position X: 5 bits (bits 25-29)
    /// - Position Y: 5 bits (bits 20-24)
    /// - Position Z: 5 bits (bits 15-19)
    /// - Model ID: 8 bits (bits 7-14)
    /// - Vertex Index: 5 bits (bits 2-6)
    /// - AO: 2 bits (bits 0-1)
    pub fn new(
        local_pos: USizeVec3,
        model_id: ModelId,
        vertex_index: u8,
        ao: u32,
        texture_index: u32,
    ) -> Self {
        Self {
            data: ((local_pos.x as u32) << 25)
                | ((local_pos.y as u32) << 20)
                | ((local_pos.z as u32) << 15)
                | ((model_id.0 as u32) << 7)
                | ((vertex_index as u32) << 2)
                | ao,
            texture_index,
        }
    }
}
