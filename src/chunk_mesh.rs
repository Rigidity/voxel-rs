use bevy::{
    asset::RenderAssetUsages,
    mesh::{Indices, PrimitiveTopology},
    prelude::*,
};

use crate::{
    ATTRIBUTE_PACKED_DATA, ATTRIBUTE_TEXTURE_INDEX, BlockFace, CHUNK_SIZE, Registry, RelevantChunks,
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

                let left = data.get_block(world_pos - IVec3::X).is_none();
                let right = data.get_block(world_pos + IVec3::X).is_none();
                let front = data.get_block(world_pos + IVec3::Z).is_none();
                let back = data.get_block(world_pos - IVec3::Z).is_none();
                let top = data.get_block(world_pos + IVec3::Y).is_none();
                let bottom = data.get_block(world_pos - IVec3::Y).is_none();

                let x = x as u32;
                let y = y as u32;
                let z = z as u32;

                // Front face (+Z)
                if front {
                    let texture_index =
                        block
                            .kind
                            .texture_index(block.data, registry, BlockFace::Front);

                    let index = mesh.index();

                    let ao0 = calculate_ao(data, world_pos, 1, 1, 1, IVec3::Z);
                    let ao1 = calculate_ao(data, world_pos, -1, 1, 1, IVec3::Z);
                    let ao2 = calculate_ao(data, world_pos, -1, -1, 1, IVec3::Z);
                    let ao3 = calculate_ao(data, world_pos, 1, -1, 1, IVec3::Z);

                    mesh.vertices.push(ChunkVertex::new(
                        [x + 1, y + 1, z + 1],
                        [1, 0],
                        [0, 0, 1],
                        ao0,
                        texture_index,
                    ));
                    mesh.vertices.push(ChunkVertex::new(
                        [x, y + 1, z + 1],
                        [0, 0],
                        [0, 0, 1],
                        ao1,
                        texture_index,
                    ));
                    mesh.vertices.push(ChunkVertex::new(
                        [x, y, z + 1],
                        [0, 1],
                        [0, 0, 1],
                        ao2,
                        texture_index,
                    ));
                    mesh.vertices.push(ChunkVertex::new(
                        [x + 1, y, z + 1],
                        [1, 1],
                        [0, 0, 1],
                        ao3,
                        texture_index,
                    ));

                    if ao0 + ao2 < ao1 + ao3 {
                        mesh.indices.extend_from_slice(&[
                            index,
                            index + 1,
                            index + 3,
                            index + 1,
                            index + 2,
                            index + 3,
                        ]);
                    } else {
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

                // Back face (-Z)
                if back {
                    let texture_index =
                        block
                            .kind
                            .texture_index(block.data, registry, BlockFace::Back);

                    let index = mesh.index();

                    let ao0 = calculate_ao(data, world_pos, -1, 1, -1, -IVec3::Z);
                    let ao1 = calculate_ao(data, world_pos, 1, 1, -1, -IVec3::Z);
                    let ao2 = calculate_ao(data, world_pos, 1, -1, -1, -IVec3::Z);
                    let ao3 = calculate_ao(data, world_pos, -1, -1, -1, -IVec3::Z);

                    mesh.vertices.push(ChunkVertex::new(
                        [x, y + 1, z],
                        [1, 0],
                        [0, 0, -1],
                        ao0,
                        texture_index,
                    ));
                    mesh.vertices.push(ChunkVertex::new(
                        [x + 1, y + 1, z],
                        [0, 0],
                        [0, 0, -1],
                        ao1,
                        texture_index,
                    ));
                    mesh.vertices.push(ChunkVertex::new(
                        [x + 1, y, z],
                        [0, 1],
                        [0, 0, -1],
                        ao2,
                        texture_index,
                    ));
                    mesh.vertices.push(ChunkVertex::new(
                        [x, y, z],
                        [1, 1],
                        [0, 0, -1],
                        ao3,
                        texture_index,
                    ));

                    if ao0 + ao2 < ao1 + ao3 {
                        mesh.indices.extend_from_slice(&[
                            index,
                            index + 1,
                            index + 3,
                            index + 1,
                            index + 2,
                            index + 3,
                        ]);
                    } else {
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

                // Left face (-X)
                if left {
                    let texture_index =
                        block
                            .kind
                            .texture_index(block.data, registry, BlockFace::Left);

                    let index = mesh.index();

                    let ao0 = calculate_ao(data, world_pos, -1, 1, 1, -IVec3::X);
                    let ao1 = calculate_ao(data, world_pos, -1, 1, -1, -IVec3::X);
                    let ao2 = calculate_ao(data, world_pos, -1, -1, -1, -IVec3::X);
                    let ao3 = calculate_ao(data, world_pos, -1, -1, 1, -IVec3::X);

                    mesh.vertices.push(ChunkVertex::new(
                        [x, y + 1, z + 1],
                        [1, 0],
                        [-1, 0, 0],
                        ao0,
                        texture_index,
                    ));
                    mesh.vertices.push(ChunkVertex::new(
                        [x, y + 1, z],
                        [0, 0],
                        [-1, 0, 0],
                        ao1,
                        texture_index,
                    ));
                    mesh.vertices.push(ChunkVertex::new(
                        [x, y, z],
                        [0, 1],
                        [-1, 0, 0],
                        ao2,
                        texture_index,
                    ));
                    mesh.vertices.push(ChunkVertex::new(
                        [x, y, z + 1],
                        [1, 1],
                        [-1, 0, 0],
                        ao3,
                        texture_index,
                    ));

                    if ao0 + ao2 < ao1 + ao3 {
                        mesh.indices.extend_from_slice(&[
                            index,
                            index + 1,
                            index + 3,
                            index + 1,
                            index + 2,
                            index + 3,
                        ]);
                    } else {
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

                // Right face (+X)
                if right {
                    let texture_index =
                        block
                            .kind
                            .texture_index(block.data, registry, BlockFace::Right);

                    let index = mesh.index();

                    let ao0 = calculate_ao(data, world_pos, 1, 1, -1, IVec3::X);
                    let ao1 = calculate_ao(data, world_pos, 1, 1, 1, IVec3::X);
                    let ao2 = calculate_ao(data, world_pos, 1, -1, 1, IVec3::X);
                    let ao3 = calculate_ao(data, world_pos, 1, -1, -1, IVec3::X);

                    mesh.vertices.push(ChunkVertex::new(
                        [x + 1, y + 1, z],
                        [1, 0],
                        [1, 0, 0],
                        ao0,
                        texture_index,
                    ));
                    mesh.vertices.push(ChunkVertex::new(
                        [x + 1, y + 1, z + 1],
                        [0, 0],
                        [1, 0, 0],
                        ao1,
                        texture_index,
                    ));
                    mesh.vertices.push(ChunkVertex::new(
                        [x + 1, y, z + 1],
                        [0, 1],
                        [1, 0, 0],
                        ao2,
                        texture_index,
                    ));
                    mesh.vertices.push(ChunkVertex::new(
                        [x + 1, y, z],
                        [1, 1],
                        [1, 0, 0],
                        ao3,
                        texture_index,
                    ));

                    if ao0 + ao2 < ao1 + ao3 {
                        mesh.indices.extend_from_slice(&[
                            index,
                            index + 1,
                            index + 3,
                            index + 1,
                            index + 2,
                            index + 3,
                        ]);
                    } else {
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

                // Top face (+Y)
                if top {
                    let texture_index =
                        block
                            .kind
                            .texture_index(block.data, registry, BlockFace::Top);

                    let index = mesh.index();

                    let ao0 = calculate_ao(data, world_pos, 1, 1, 1, IVec3::Y);
                    let ao1 = calculate_ao(data, world_pos, 1, 1, -1, IVec3::Y);
                    let ao2 = calculate_ao(data, world_pos, -1, 1, -1, IVec3::Y);
                    let ao3 = calculate_ao(data, world_pos, -1, 1, 1, IVec3::Y);

                    mesh.vertices.push(ChunkVertex::new(
                        [x + 1, y + 1, z + 1],
                        [1, 1],
                        [0, 1, 0],
                        ao0,
                        texture_index,
                    ));
                    mesh.vertices.push(ChunkVertex::new(
                        [x + 1, y + 1, z],
                        [1, 0],
                        [0, 1, 0],
                        ao1,
                        texture_index,
                    ));
                    mesh.vertices.push(ChunkVertex::new(
                        [x, y + 1, z],
                        [0, 0],
                        [0, 1, 0],
                        ao2,
                        texture_index,
                    ));
                    mesh.vertices.push(ChunkVertex::new(
                        [x, y + 1, z + 1],
                        [0, 1],
                        [0, 1, 0],
                        ao3,
                        texture_index,
                    ));

                    if ao0 + ao2 < ao1 + ao3 {
                        mesh.indices.extend_from_slice(&[
                            index,
                            index + 1,
                            index + 3,
                            index + 1,
                            index + 2,
                            index + 3,
                        ]);
                    } else {
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

                // Bottom face (-Y)
                if bottom {
                    let texture_index =
                        block
                            .kind
                            .texture_index(block.data, registry, BlockFace::Bottom);

                    let index = mesh.index();

                    let ao0 = calculate_ao(data, world_pos, 1, -1, -1, -IVec3::Y);
                    let ao1 = calculate_ao(data, world_pos, 1, -1, 1, -IVec3::Y);
                    let ao2 = calculate_ao(data, world_pos, -1, -1, 1, -IVec3::Y);
                    let ao3 = calculate_ao(data, world_pos, -1, -1, -1, -IVec3::Y);

                    mesh.vertices.push(ChunkVertex::new(
                        [x + 1, y, z],
                        [1, 1],
                        [0, -1, 0],
                        ao0,
                        texture_index,
                    ));
                    mesh.vertices.push(ChunkVertex::new(
                        [x + 1, y, z + 1],
                        [1, 0],
                        [0, -1, 0],
                        ao1,
                        texture_index,
                    ));
                    mesh.vertices.push(ChunkVertex::new(
                        [x, y, z + 1],
                        [0, 0],
                        [0, -1, 0],
                        ao2,
                        texture_index,
                    ));
                    mesh.vertices.push(ChunkVertex::new(
                        [x, y, z],
                        [0, 1],
                        [0, -1, 0],
                        ao3,
                        texture_index,
                    ));

                    if ao0 + ao2 < ao1 + ao3 {
                        mesh.indices.extend_from_slice(&[
                            index,
                            index + 1,
                            index + 3,
                            index + 1,
                            index + 2,
                            index + 3,
                        ]);
                    } else {
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
    }

    mesh.build()
}

fn calculate_ao(
    data: &RelevantChunks,
    block_pos: IVec3,
    dx: i32,
    dy: i32,
    dz: i32,
    normal: IVec3,
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

    let side1 = data.get_block(side1_pos).is_some();
    let side2 = data.get_block(side2_pos).is_some();
    let corner = data.get_block(corner_pos).is_some();

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
    pub fn new(
        position: [u32; 3],
        tex_coords: [u32; 2],
        _normal: [i32; 3],
        ao: u32,
        texture_index: u32,
    ) -> Self {
        Self {
            data: (position[0] << 26)
                | (position[1] << 20)
                | (position[2] << 14)
                | (tex_coords[0] << 13)
                | (tex_coords[1] << 12)
                | (ao << 10),
            texture_index,
        }
    }
}
