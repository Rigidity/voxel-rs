use crate::{Block, Vertex, VoxelMesh};

pub const CHUNK_SIZE: usize = 16;

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

                    let index = mesh.index();

                    mesh.vertices.push(Vertex::new(
                        [x - 0.0868241, y + 0.49240386, z + 0.0],
                        [0.4131759, 0.00759614],
                    ));

                    mesh.vertices.push(Vertex::new(
                        [x - 0.49513406, y + 0.06958647, z + 0.0],
                        [0.0048659444, 0.43041354],
                    ));

                    mesh.vertices.push(Vertex::new(
                        [x - 0.21918549, y - 0.44939706, z + 0.0],
                        [0.28081453, 0.949397],
                    ));

                    mesh.vertices.push(Vertex::new(
                        [x + 0.35966998, y - 0.3473291, z + 0.0],
                        [0.85967, 0.84732914],
                    ));

                    mesh.vertices.push(Vertex::new(
                        [x + 0.44147372, y + 0.2347359, z + 0.0],
                        [0.9414737, 0.2652641],
                    ));

                    mesh.indices.extend_from_slice(&[
                        index,
                        index + 1,
                        index + 4,
                        index + 1,
                        index + 2,
                        index + 4,
                        index + 2,
                        index + 3,
                        index + 4,
                    ]);
                }
            }
        }

        mesh
    }
}
