use glam::Vec3;

use crate::{Aabb, Registry, TextureArrayBuilder};

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

    fn id(&self, registry: &Registry) -> u16 {
        registry.block_id(self.base_name())
    }

    fn register_textures(&self, builder: &mut TextureArrayBuilder, registry: &mut Registry);

    fn get_aabb(&self, _data: u64) -> Option<Aabb> {
        Some(Aabb::new(Vec3::ZERO, Vec3::splat(1.0)))
    }

    fn texture_index(&self, _data: u64, registry: &Registry) -> u32;
}

pub struct Dirt;

pub static DIRT: &dyn BlockType = &Dirt;

impl BlockType for Dirt {
    fn base_name(&self) -> &'static str {
        "dirt"
    }

    fn register_textures(&self, builder: &mut TextureArrayBuilder, registry: &mut Registry) {
        let id = self.id(registry);
        let texture_index = builder.add_bytes(include_bytes!("../textures/Dirt.png"));
        registry.register_texture(id, 0, texture_index);
    }

    fn texture_index(&self, _data: u64, registry: &Registry) -> u32 {
        let id = self.id(registry);
        registry.texture_index(id, 0)
    }
}

pub struct Rock;

pub static ROCK: &dyn BlockType = &Rock;

impl BlockType for Rock {
    fn base_name(&self) -> &'static str {
        "rock"
    }

    fn register_textures(&self, builder: &mut TextureArrayBuilder, registry: &mut Registry) {
        let id = self.id(registry);
        let texture_index = builder.add_bytes(include_bytes!("../textures/Rock.png"));
        registry.register_texture(id, 0, texture_index);
    }

    fn texture_index(&self, _data: u64, registry: &Registry) -> u32 {
        let id = self.id(registry);
        registry.texture_index(id, 0)
    }
}
