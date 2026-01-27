use glam::Vec3;
use image::{ColorType, DynamicImage, GenericImage, Rgba};

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
        let id = registry.block_id(self.base_name());
        let texture_index = builder.add_bytes(include_bytes!("../textures/Dirt.png"));
        registry.register_texture(id, 0, texture_index);
    }

    fn texture_index(&self, _data: u64, registry: &Registry) -> u32 {
        let id = registry.block_id(self.base_name());
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
        let id = registry.block_id(self.base_name());
        let texture_index = builder.add_bytes(include_bytes!("../textures/Rock.png"));
        registry.register_texture(id, 0, texture_index);
    }

    fn texture_index(&self, _data: u64, registry: &Registry) -> u32 {
        let id = registry.block_id(self.base_name());
        registry.texture_index(id, 0)
    }
}

pub struct Test;

pub static TEST: &dyn BlockType = &Test;

impl BlockType for Test {
    fn base_name(&self) -> &'static str {
        "test"
    }

    fn register_textures(&self, builder: &mut TextureArrayBuilder, registry: &mut Registry) {
        let id = registry.block_id(self.base_name());

        let mut image = DynamicImage::new(builder.width(), builder.height(), ColorType::Rgba8);

        for x in 0..builder.width() {
            for y in 0..builder.height() {
                let color = ((x as f32 + y as f32) / 2.0 / (builder.width() as f32 - 1.0) * 255.0)
                    .clamp(0.0, 255.0) as u8;
                image.put_pixel(x, y, Rgba([color, 0, color, 255]));
            }
        }

        let texture_index = builder.add_image(image);
        registry.register_texture(id, 0, texture_index);
    }

    fn texture_index(&self, _data: u64, registry: &Registry) -> u32 {
        let id = registry.block_id(self.base_name());
        registry.texture_index(id, 0)
    }
}
