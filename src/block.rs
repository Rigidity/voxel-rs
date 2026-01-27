use glam::Vec3;
use strum::{Display, EnumIter};

use crate::{Aabb, Registry, TextureArrayBuilder};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Block {
    pub kind: BlockKind,
    pub data: u64,
}

impl Block {
    pub fn new(kind: BlockKind, data: u64) -> Self {
        Self { kind, data }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumIter, Display)]
#[repr(u16)]
pub enum BlockKind {
    #[strum(to_string = "Dirt")]
    Dirt,

    #[strum(to_string = "Rock")]
    Rock,
}

impl BlockKind {
    pub fn get_aabb(&self, _data: u64) -> Option<Aabb> {
        Some(Aabb::new(Vec3::ZERO, Vec3::splat(1.0)))
    }

    pub fn register_textures(&self, builder: &mut TextureArrayBuilder, registry: &mut Registry) {
        match self {
            Self::Dirt => {
                let texture_index = builder.add_bytes(include_bytes!("../textures/Dirt.png"));
                registry.register_texture(*self, 0, texture_index);
            }
            Self::Rock => {
                let texture_index = builder.add_bytes(include_bytes!("../textures/Rock.png"));
                registry.register_texture(*self, 0, texture_index);
            }
        }
    }

    pub fn texture_index(&self, _data: u64, registry: &Registry) -> u32 {
        match self {
            Self::Dirt => registry.texture_index(*self, 0),
            Self::Rock => registry.texture_index(*self, 0),
        }
    }
}
