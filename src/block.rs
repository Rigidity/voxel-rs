use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{Aabb, BlockId, ModelId, PackedData, Registry};

pub trait BlockType: 'static + Send + Sync {
    fn unique_name(&self) -> String;

    fn get_aabb(&self, _data: PackedData) -> Option<Aabb> {
        Some(Aabb::new(Vec3::ZERO, Vec3::splat(1.0)))
    }

    fn register(&self, registry: &mut Registry);

    fn face_data(&self, _face: BlockFace, data: PackedData) -> PackedData {
        data
    }

    fn is_solid(&self) -> bool {
        true
    }

    /// Returns the model ID and vertex indices for a given face
    /// Default implementation returns the standard cube model with 4 vertices per face
    fn get_face_vertices(&self, face: BlockFace, _model_id: ModelId) -> (ModelId, [u8; 4]) {
        let base_index = match face {
            BlockFace::Front => 0,
            BlockFace::Back => 4,
            BlockFace::Left => 8,
            BlockFace::Right => 12,
            BlockFace::Top => 16,
            BlockFace::Bottom => 20,
        };
        (_model_id, [base_index, base_index + 1, base_index + 2, base_index + 3])
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BlockFace {
    Top,
    Bottom,
    Left,
    Right,
    Front,
    Back,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Block {
    pub id: BlockId,
    pub data: PackedData,
}

impl Block {
    pub fn new(id: BlockId, data: PackedData) -> Self {
        Self { id, data }
    }
}
