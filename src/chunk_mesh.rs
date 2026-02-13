use bevy::{
    asset::RenderAssetUsages,
    mesh::{Indices, PrimitiveTopology},
    prelude::*,
};

use crate::{
    ATTRIBUTE_PACKED_DATA, ATTRIBUTE_TEXTURE_INDEX, Block, BlockFace, CHUNK_SIZE, ModelId,
    Registry, RelevantChunks,
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

                let solid = registry.block_type(block.id).is_solid();

                let is_transparent = |neighboring_block: Block| -> bool {
                    let neighboring_solid = registry.block_type(neighboring_block.id).is_solid();
                    !neighboring_solid && solid
                };

                let left = data
                    .get_block(world_pos - IVec3::X)
                    .is_none_or(is_transparent);
                let right = data
                    .get_block(world_pos + IVec3::X)
                    .is_none_or(is_transparent);
                let front = data
                    .get_block(world_pos + IVec3::Z)
                    .is_none_or(is_transparent);
                let back = data
                    .get_block(world_pos - IVec3::Z)
                    .is_none_or(is_transparent);
                let top = data
                    .get_block(world_pos + IVec3::Y)
                    .is_none_or(is_transparent);
                let bottom = data
                    .get_block(world_pos - IVec3::Y)
                    .is_none_or(is_transparent);

                let cube_model_id = registry
                    .model_registry
                    .get_model_id("cube")
                    .expect("Cube model not registered");

                let x = x as u32;
                let y = y as u32;
                let z = z as u32;

                // Front face (+Z)
                if front {
                    add_face(
                        &mut mesh,
                        data,
                        world_pos,
                        [x, y, z],
                        BlockFace::Front,
                        block,
                        cube_model_id,
                        solid,
                        registry,
                    );
                }

                // Back face (-Z)
                if back {
                    add_face(
                        &mut mesh,
                        data,
                        world_pos,
                        [x, y, z],
                        BlockFace::Back,
                        block,
                        cube_model_id,
                        solid,
                        registry,
                    );
                }

                // Left face (-X)
                if left {
                    add_face(
                        &mut mesh,
                        data,
                        world_pos,
                        [x, y, z],
                        BlockFace::Left,
                        block,
                        cube_model_id,
                        solid,
                        registry,
                    );
                }

                // Right face (+X)
                if right {
                    add_face(
                        &mut mesh,
                        data,
                        world_pos,
                        [x, y, z],
                        BlockFace::Right,
                        block,
                        cube_model_id,
                        solid,
                        registry,
                    );
                }

                // Top face (+Y)
                if top {
                    add_face(
                        &mut mesh,
                        data,
                        world_pos,
                        [x, y, z],
                        BlockFace::Top,
                        block,
                        cube_model_id,
                        solid,
                        registry,
                    );
                }

                // Bottom face (-Y)
                if bottom {
                    add_face(
                        &mut mesh,
                        data,
                        world_pos,
                        [x, y, z],
                        BlockFace::Bottom,
                        block,
                        cube_model_id,
                        solid,
                        registry,
                    );
                }
            }
        }
    }

    mesh.build()
}

#[allow(clippy::too_many_arguments)]
fn add_face(
    mesh: &mut ChunkMeshBuilder,
    data: &RelevantChunks,
    world_pos: IVec3,
    block_pos: [u32; 3],
    face: BlockFace,
    block: Block,
    default_model_id: ModelId,
    solid: bool,
    registry: &Registry,
) {
    let texture_index = registry.texture_index(block, face);
    let block_type = registry.block_type(block.id);
    let (model_id, vertex_indices) = block_type.get_face_vertices(face, default_model_id);

    let index = mesh.index();

    // Calculate AO for each vertex
    let (normal, ao_offsets) = match face {
        BlockFace::Front => (IVec3::Z, [(1, 1, 1), (-1, 1, 1), (-1, -1, 1), (1, -1, 1)]),
        BlockFace::Back => (
            -IVec3::Z,
            [(-1, 1, -1), (1, 1, -1), (1, -1, -1), (-1, -1, -1)],
        ),
        BlockFace::Left => (
            -IVec3::X,
            [(-1, 1, 1), (-1, 1, -1), (-1, -1, -1), (-1, -1, 1)],
        ),
        BlockFace::Right => (IVec3::X, [(1, 1, -1), (1, 1, 1), (1, -1, 1), (1, -1, -1)]),
        BlockFace::Top => (IVec3::Y, [(1, 1, 1), (1, 1, -1), (-1, 1, -1), (-1, 1, 1)]),
        BlockFace::Bottom => (
            -IVec3::Y,
            [(1, -1, -1), (1, -1, 1), (-1, -1, 1), (-1, -1, -1)],
        ),
    };

    let aos: [u32; 4] = [
        calculate_ao(
            data,
            world_pos,
            ao_offsets[0].0,
            ao_offsets[0].1,
            ao_offsets[0].2,
            normal,
            registry,
        ),
        calculate_ao(
            data,
            world_pos,
            ao_offsets[1].0,
            ao_offsets[1].1,
            ao_offsets[1].2,
            normal,
            registry,
        ),
        calculate_ao(
            data,
            world_pos,
            ao_offsets[2].0,
            ao_offsets[2].1,
            ao_offsets[2].2,
            normal,
            registry,
        ),
        calculate_ao(
            data,
            world_pos,
            ao_offsets[3].0,
            ao_offsets[3].1,
            ao_offsets[3].2,
            normal,
            registry,
        ),
    ];

    // Add vertices
    for i in 0..4 {
        mesh.vertices.push(ChunkVertex::new(
            block_pos,
            model_id,
            vertex_indices[i],
            aos[i],
            texture_index,
        ));
    }

    // Add indices with proper winding based on AO
    if aos[0] + aos[2] < aos[1] + aos[3] {
        mesh.indices.extend_from_slice(&[
            index,
            index + 1,
            index + 3,
            index + 1,
            index + 2,
            index + 3,
        ]);
        // Add back face for non-solid blocks
        if !solid {
            mesh.indices.extend_from_slice(&[
                index + 3,
                index + 1,
                index,
                index + 3,
                index + 2,
                index + 1,
            ]);
        }
    } else {
        mesh.indices
            .extend_from_slice(&[index, index + 1, index + 2, index + 2, index + 3, index]);
        // Add back face for non-solid blocks
        if !solid {
            mesh.indices.extend_from_slice(&[
                index,
                index + 3,
                index + 2,
                index + 2,
                index + 1,
                index,
            ]);
        }
    }
}

fn calculate_ao(
    data: &RelevantChunks,
    block_pos: IVec3,
    dx: i32,
    dy: i32,
    dz: i32,
    normal: IVec3,
    registry: &Registry,
) -> u32 {
    let vertex_offset = IVec3::new(dx, dy, dz);

    let (axis1, axis2) = if normal.x.abs() == 1 {
        (IVec3::Y, IVec3::Z)
    } else if normal.y.abs() == 1 {
        (IVec3::X, IVec3::Z)
    } else {
        (IVec3::X, IVec3::Y)
    };

    let side1_dir = vertex_offset.dot(axis1).signum();
    let side2_dir = vertex_offset.dot(axis2).signum();

    let side1_pos = block_pos + normal + axis1 * side1_dir;
    let side2_pos = block_pos + normal + axis2 * side2_dir;
    let corner_pos = block_pos + normal + axis1 * side1_dir + axis2 * side2_dir;

    let side1 = data
        .get_block(side1_pos)
        .is_some_and(|block| registry.block_type(block.id).is_solid());
    let side2 = data
        .get_block(side2_pos)
        .is_some_and(|block| registry.block_type(block.id).is_solid());
    let corner = data
        .get_block(corner_pos)
        .is_some_and(|block| registry.block_type(block.id).is_solid());

    let occlusion = if side1 && side2 {
        3
    } else {
        (side1 as u32) + (side2 as u32) + (corner as u32)
    };

    3 - occlusion
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
        position: [u32; 3],
        model_id: ModelId,
        vertex_index: u8,
        ao: u32,
        texture_index: u32,
    ) -> Self {
        Self {
            data: (position[0] << 25)
                | (position[1] << 20)
                | (position[2] << 15)
                | ((model_id.0 as u32) << 7)
                | ((vertex_index as u32) << 2)
                | ao,
            texture_index,
        }
    }
}
