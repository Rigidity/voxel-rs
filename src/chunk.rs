use glam::USizeVec3;

use crate::{Block, ChunkData};

#[derive(Debug, Clone)]
pub struct Chunk {
    data: ChunkData,
    mesh_status: MeshStatus,
}

impl Chunk {
    pub fn new(data: ChunkData) -> Self {
        Self {
            data,
            mesh_status: MeshStatus::Queued,
        }
    }

    pub fn data(&self) -> &ChunkData {
        &self.data
    }

    pub fn data_mut(&mut self) -> &mut ChunkData {
        &mut self.data
    }

    pub fn get_block(&self, local_pos: USizeVec3) -> Option<Block> {
        self.data.get_block(local_pos)
    }

    pub fn set_block(&mut self, local_pos: USizeVec3, block: Option<Block>) {
        self.data.set_block(local_pos, block);
    }

    pub fn mesh_status(&self) -> MeshStatus {
        self.mesh_status
    }

    pub fn queue_remesh(&mut self) {
        if self.mesh_status != MeshStatus::Urgent {
            self.mesh_status = MeshStatus::Queued;
        }
    }

    pub fn queue_urgent_remesh(&mut self) {
        self.mesh_status = MeshStatus::Urgent;
    }

    pub fn set_mesh_complete(&mut self) {
        self.mesh_status = MeshStatus::Complete;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MeshStatus {
    /// The chunk already has a mesh and hasn't been modified.
    Complete,

    /// The chunk is visible and should be meshed based on distance from the player.
    Queued,

    /// The chunk should be remeshed first.
    Urgent,
}
