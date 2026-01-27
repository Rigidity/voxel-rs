use glam::Vec3;
use num_derive::{FromPrimitive, ToPrimitive};
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
                let base_image =
                    image::load_from_memory(include_bytes!("../textures/Soil.png")).unwrap();
                let overlay_image =
                    image::load_from_memory(include_bytes!("../textures/GrassOverlay.png"))
                        .unwrap();

                for material in Material::iter() {
                    if material.kind() == MaterialKind::Soil {
                        let mut base_image = base_image.clone().into_rgba8();

                        material.color_image(&mut base_image);

                        let texture_index = builder.add_image(base_image.clone().into());

                        registry.register_texture(
                            *self,
                            SoilData {
                                material,
                                grass_material: None,
                            }
                            .encode(),
                            texture_index,
                        );

                        for grass_material in Material::iter() {
                            if grass_material.kind() == MaterialKind::Grass {
                                let mut overlay_image = overlay_image.clone().into_rgba8();
                                grass_material.color_image(&mut overlay_image);

                                let mut composite_image = base_image.clone();

                                for x in 0..overlay_image.width() {
                                    for y in 0..overlay_image.height() {
                                        let pixel = overlay_image.get_pixel(x, y);
                                        if pixel.0[3] > 0 {
                                            composite_image.put_pixel(x, y, *pixel);
                                        }
                                    }
                                }

                                let texture_index = builder.add_image(composite_image.into());

                                registry.register_texture(
                                    *self,
                                    SoilData {
                                        material,
                                        grass_material: Some(grass_material),
                                    }
                                    .encode(),
                                    texture_index,
                                );
                            }
                        }
                    }
                }
            }
            Self::Rock => {
                for rock_type in RockType::iter() {
                    let image = match rock_type {
                        RockType::Rock => {
                            image::load_from_memory(include_bytes!("../textures/Rock.png")).unwrap()
                        }
                        RockType::Stone => {
                            image::load_from_memory(include_bytes!("../textures/Stone.png"))
                                .unwrap()
                        }
                    };

                    for material in Material::iter() {
                        if material.kind() == MaterialKind::Rock {
                            let mut image = image.clone().into_rgba8();

                            material.color_image(&mut image);

                            let texture_index = builder.add_image(image.into());

                            registry.register_texture(
                                *self,
                                RockData {
                                    rock_type,
                                    material,
                                }
                                .encode(),
                                texture_index,
                            );
                        }
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
    pub rock_type: RockType,
    pub material: Material,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Display, EnumIter, ToPrimitive, FromPrimitive,
)]
pub enum RockType {
    #[strum(to_string = "Rock")]
    Rock,

    #[strum(to_string = "Stone")]
    Stone,
}

impl BlockData for RockData {
    fn encode(&self) -> u64 {
        let rock_type = self.rock_type.to_u8().unwrap() as u64;
        let material = self.material.to_u16().unwrap() as u64;

        rock_type | (material << 8)
    }

    fn decode(data: u64) -> Self {
        let rock_type = RockType::from_u8((data & 0xFF) as u8).unwrap();
        let material = Material::from_u16(((data >> 8) & 0xFFFF) as u16).unwrap();

        Self {
            rock_type,
            material,
        }
    }
}

pub struct SoilData {
    pub material: Material,
    pub grass_material: Option<Material>,
}

impl BlockData for SoilData {
    fn encode(&self) -> u64 {
        let material = self.material.to_u16().unwrap() as u64;
        let grass_material = self
            .grass_material
            .map(|m| m.to_u16().unwrap())
            .unwrap_or(u16::MAX) as u64;

        material | (grass_material << 16)
    }

    fn decode(data: u64) -> Self {
        let material = Material::from_u16((data & 0xFFFF) as u16).unwrap();
        let grass_material_value = ((data >> 16) & 0xFFFF) as u16;
        let grass_material = if grass_material_value == u16::MAX {
            None
        } else {
            Some(Material::from_u16(grass_material_value).unwrap())
        };

        Self {
            material,
            grass_material,
        }
    }
}
