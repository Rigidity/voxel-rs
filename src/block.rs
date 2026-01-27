use glam::Vec3;

use crate::Aabb;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Block {
    pub id: u16,
    pub data: u64,
}

impl Block {
    pub fn new(id: u16, data: u64) -> Self {
        Self { id, data }
    }
}

pub trait BlockType: Send + Sync + 'static {
    fn base_name(&self) -> &'static str;

    fn get_aabb(&self, _data: u64) -> Option<Aabb> {
        Some(Aabb::new(Vec3::ZERO, Vec3::splat(1.0)))
    }

    fn texture_index(&self, _data: u64) -> u32 {
        0
    }
}

pub struct Dirt;

impl BlockType for Dirt {
    fn base_name(&self) -> &'static str {
        "dirt"
    }

    fn texture_index(&self, _data: u64) -> u32 {
        0
    }
}

pub struct Rock;

impl BlockType for Rock {
    fn base_name(&self) -> &'static str {
        "rock"
    }

    fn texture_index(&self, _data: u64) -> u32 {
        1
    }
}
