use glam::Vec3;
use num_traits::{FromPrimitive, ToPrimitive};
use strum::{Display, EnumIter, IntoEnumIterator};

use crate::{Aabb, Material, MaterialKind, Registry, TextureArrayBuilder};

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
    #[strum(to_string = "Soil")]
    Soil,

    #[strum(to_string = "Rock")]
    Rock,
}

impl BlockKind {
    pub fn get_aabb(&self, _data: u64) -> Option<Aabb> {
        Some(Aabb::new(Vec3::ZERO, Vec3::splat(1.0)))
    }

    pub fn register_textures(&self, builder: &mut TextureArrayBuilder, registry: &mut Registry) {
        match self {
            Self::Soil => {
                let image =
                    image::load_from_memory(include_bytes!("../textures/Soil.png")).unwrap();

                for material in Material::iter() {
                    if material.kind() == MaterialKind::Soil {
                        let mut image = image.clone().into_rgba8();

                        material.color_image(&mut image);

                        let texture_index = builder.add_image(image.into());

                        registry.register_texture(
                            *self,
                            SoilData { material }.encode(),
                            texture_index,
                        );
                    }
                }
            }
            Self::Rock => {
                let image =
                    image::load_from_memory(include_bytes!("../textures/Rock.png")).unwrap();

                for material in Material::iter() {
                    if material.kind() == MaterialKind::Rock {
                        let mut image = image.clone().into_rgba8();

                        material.color_image(&mut image);

                        let texture_index = builder.add_image(image.into());

                        registry.register_texture(
                            *self,
                            RockData { material }.encode(),
                            texture_index,
                        );
                    }
                }
            }
        }
    }

    pub fn texture_index(&self, data: u64, registry: &Registry) -> u32 {
        match self {
            Self::Soil => registry.texture_index(*self, data),
            Self::Rock => registry.texture_index(*self, data),
        }
    }
}

pub trait BlockData {
    fn encode(&self) -> u64;
    fn decode(data: u64) -> Self;
}

pub struct RockData {
    pub material: Material,
}

impl BlockData for RockData {
    fn encode(&self) -> u64 {
        let material = self.material.to_u16().unwrap();
        material as u64
    }

    fn decode(data: u64) -> Self {
        let material = Material::from_u16(data as u16).unwrap();
        Self { material }
    }
}

pub struct SoilData {
    pub material: Material,
}

impl BlockData for SoilData {
    fn encode(&self) -> u64 {
        let material = self.material.to_u16().unwrap();
        material as u64
    }

    fn decode(data: u64) -> Self {
        let material = Material::from_u16(data as u16).unwrap();
        Self { material }
    }
}
