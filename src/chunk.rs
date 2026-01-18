use crate::{Block, Vertex, VoxelMesh};

pub const CHUNK_SIZE: usize = 32;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Chunk {
    blocks: [Block; CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE],
}

impl Default for Chunk {
    fn default() -> Self {
        Self {
            blocks: [Block::Rock; CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE],
        }
    }
}

impl Chunk {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_block(&self, x: usize, y: usize, z: usize) -> Block {
        self.blocks[x + y * CHUNK_SIZE + z * CHUNK_SIZE * CHUNK_SIZE]
    }

    pub fn set_block(&mut self, x: usize, y: usize, z: usize, block: Block) {
        self.blocks[x + y * CHUNK_SIZE + z * CHUNK_SIZE * CHUNK_SIZE] = block;
    }

    pub fn create_mesh(&self) -> VoxelMesh {
        let mut mesh = VoxelMesh::new();

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

        mesh
    }
}
