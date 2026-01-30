use bevy::prelude::*;
use image::{DynamicImage, GenericImageView};
use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::{FromPrimitive, ToPrimitive};
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use strum::{Display, EnumIter, IntoEnumIterator};

use crate::{Aabb, Material, MaterialKind, Registry, TextureArrayBuilder};

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
    pub kind: BlockKind,
    pub data: u64,
}

impl Block {
    pub fn new(kind: BlockKind, data: u64) -> Self {
        Self { kind, data }
    }
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, EnumIter, Display, Serialize_repr, Deserialize_repr,
)]
#[repr(u16)]
pub enum BlockKind {
    #[strum(to_string = "Soil")]
    Soil,

    #[strum(to_string = "Rock")]
    Rock,

    #[strum(to_string = "Wood Log")]
    WoodLog,
}

impl BlockKind {
    pub fn get_aabb(&self, _data: u64) -> Option<Aabb> {
        Some(Aabb::new(Vec3::ZERO, Vec3::splat(1.0)))
    }

    pub fn register_textures(
        &self,
        builder: &mut TextureArrayBuilder,
        registry: &mut Registry,
        atlas: &DynamicImage,
    ) {
        match self {
            Self::Soil => {
                let base_image: DynamicImage = atlas.view(0, 0, 16, 16).to_image().into();
                let overlay_image: DynamicImage = atlas.view(16, 0, 16, 16).to_image().into();

                for grass_material in Material::iter() {
                    if grass_material.kind() == MaterialKind::Grass {
                        let mut image = base_image.clone().into_rgba8();
                        grass_material.color_image(&mut image);

                        let texture_index = builder.add_image(image.into());

                        registry.register_texture(
                            *self,
                            SoilTextureData::Grass { grass_material }.encode(),
                            texture_index,
                        );
                    }
                }

                for material in Material::iter() {
                    if material.kind() == MaterialKind::Soil {
                        let mut base_image = base_image.clone().into_rgba8();

                        material.color_image(&mut base_image);

                        let texture_index = builder.add_image(base_image.clone().into());

                        registry.register_texture(
                            *self,
                            SoilTextureData::Soil {
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
                                    SoilTextureData::Soil {
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
                    let image: DynamicImage = match rock_type {
                        RockType::Rock => atlas.view(48, 0, 16, 16).to_image().into(),
                        RockType::Stone => atlas.view(32, 0, 16, 16).to_image().into(),
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
            Self::WoodLog => {
                let top_image: DynamicImage = atlas.view(0, 16, 16, 16).to_image().into();
                let side_image: DynamicImage = atlas.view(16, 16, 16, 16).to_image().into();

                for material in Material::iter() {
                    if material.kind() == MaterialKind::Wood {
                        let mut top_image = top_image.clone().into_rgba8();
                        material.color_image(&mut top_image);

                        let mut side_image = side_image.clone().into_rgba8();
                        material.color_image(&mut side_image);

                        let texture_index = builder.add_image(top_image.into());

                        registry.register_texture(
                            *self,
                            WoodLogTextureData {
                                wood_material: material,
                                face: WoodLogFace::TopBottom,
                            }
                            .encode(),
                            texture_index,
                        );

                        let texture_index = builder.add_image(side_image.into());

                        registry.register_texture(
                            *self,
                            WoodLogTextureData {
                                wood_material: material,
                                face: WoodLogFace::Side,
                            }
                            .encode(),
                            texture_index,
                        );
                    }
                }
            }
        }
    }

    pub fn texture_index(&self, data: u64, registry: &Registry, face: BlockFace) -> u32 {
        match self {
            Self::Soil => {
                let soil_data = SoilData::decode(data);

                let Some(grass_material) = soil_data.grass_material else {
                    return registry.texture_index(
                        *self,
                        SoilTextureData::Soil {
                            material: soil_data.material,
                            grass_material: None,
                        }
                        .encode(),
                    );
                };

                match face {
                    BlockFace::Top => registry
                        .texture_index(*self, SoilTextureData::Grass { grass_material }.encode()),
                    BlockFace::Bottom => registry.texture_index(
                        *self,
                        SoilTextureData::Soil {
                            material: soil_data.material,
                            grass_material: None,
                        }
                        .encode(),
                    ),
                    _ => registry.texture_index(
                        *self,
                        SoilTextureData::Soil {
                            material: soil_data.material,
                            grass_material: Some(grass_material),
                        }
                        .encode(),
                    ),
                }
            }
            Self::Rock => registry.texture_index(*self, data),
            Self::WoodLog => {
                let wood_log_data = WoodLogData::decode(data);

                match face {
                    BlockFace::Top | BlockFace::Bottom => registry.texture_index(
                        *self,
                        WoodLogTextureData {
                            wood_material: wood_log_data.wood_material,
                            face: WoodLogFace::TopBottom,
                        }
                        .encode(),
                    ),
                    BlockFace::Left | BlockFace::Right | BlockFace::Front | BlockFace::Back => {
                        registry.texture_index(
                            *self,
                            WoodLogTextureData {
                                wood_material: wood_log_data.wood_material,
                                face: WoodLogFace::Side,
                            }
                            .encode(),
                        )
                    }
                }
            }
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

enum SoilTextureData {
    Soil {
        material: Material,
        grass_material: Option<Material>,
    },
    Grass {
        grass_material: Material,
    },
}

impl BlockData for SoilTextureData {
    fn encode(&self) -> u64 {
        match self {
            Self::Soil {
                material,
                grass_material,
            } => {
                let discriminant: u64 = 0;
                let material = material.to_u16().unwrap() as u64;
                let grass_material = grass_material
                    .map(|m| m.to_u16().unwrap())
                    .unwrap_or(u16::MAX) as u64;

                discriminant | (material << 1) | (grass_material << 17)
            }
            Self::Grass { grass_material } => {
                let discriminant: u64 = 1;
                let grass_material = grass_material.to_u16().unwrap() as u64;

                discriminant | (grass_material << 1)
            }
        }
    }

    fn decode(data: u64) -> Self {
        let discriminant = data & 0x01;

        match discriminant {
            0 => Self::Soil {
                material: Material::from_u16((data >> 1) as u16).unwrap(),
                grass_material: Some(Material::from_u16((data >> 17) as u16).unwrap()),
            },
            1 => Self::Grass {
                grass_material: Material::from_u16((data >> 1) as u16).unwrap(),
            },
            _ => unreachable!(),
        }
    }
}

pub struct WoodLogData {
    pub wood_material: Material,
}

impl BlockData for WoodLogData {
    fn encode(&self) -> u64 {
        self.wood_material.to_u16().unwrap() as u64
    }

    fn decode(data: u64) -> Self {
        let wood_material = Material::from_u16((data & 0xFFFF) as u16).unwrap();

        Self { wood_material }
    }
}

struct WoodLogTextureData {
    wood_material: Material,
    face: WoodLogFace,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, ToPrimitive, FromPrimitive)]
#[repr(u8)]
enum WoodLogFace {
    Side,
    TopBottom,
}

impl BlockData for WoodLogTextureData {
    fn encode(&self) -> u64 {
        let wood_material = self.wood_material.to_u16().unwrap() as u64;
        let face = self.face.to_u8().unwrap() as u64;

        wood_material | (face << 16)
    }

    fn decode(data: u64) -> Self {
        let wood_material = Material::from_u16((data & 0xFFFF) as u16).unwrap();
        let face = WoodLogFace::from_u8((data >> 16) as u8).unwrap();

        Self {
            wood_material,
            face,
        }
    }
}
