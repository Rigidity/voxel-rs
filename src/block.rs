use glam::{IVec3, Vec3};

use crate::Aabb;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Block {
    #[default]
    Air,
    Rock,
}

impl Block {
    pub fn is_solid(&self) -> bool {
        !matches!(self, Block::Air)
    }

    pub fn aabb(&self, position: IVec3) -> Option<Aabb> {
        match self {
            Block::Air => None,
            Block::Rock => Some(Aabb::new(
                Vec3::new(position.x as f32, position.y as f32, position.z as f32),
                Vec3::splat(1.0),
            )),
        }
    }
}
