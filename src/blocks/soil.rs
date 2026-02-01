use image::{GenericImage, GenericImageView};
use strum::IntoEnumIterator;

use crate::{Block, BlockFace, BlockType, Material, MaterialKind, PackedData, Registry};

pub struct Soil;

impl BlockType for Soil {
    fn unique_name(&self) -> String {
        "soil".to_string()
    }

    fn register(&self, registry: &mut Registry) {
        let block_id = registry.block_id(&self.unique_name());
        let soil_image =
            image::load_from_memory(include_bytes!("../../textures/soil.png")).unwrap();
        let grass_side_image =
            image::load_from_memory(include_bytes!("../../textures/grass.png")).unwrap();

        for grass_material in Material::iter() {
            if grass_material.kind() == MaterialKind::Grass {
                let mut image = soil_image.clone();
                grass_material.color_image(image.as_mut_rgba8().unwrap());

                let texture_index = registry.add_image(image);

                registry.register_texture(
                    Block::new(
                        block_id,
                        PackedData::builder()
                            .with_bool(true)
                            .with_material(grass_material)
                            .build(),
                    ),
                    texture_index,
                );
            }
        }

        for material in Material::iter() {
            if material.kind() == MaterialKind::Soil {
                let mut image = soil_image.clone();
                material.color_image(image.as_mut_rgba8().unwrap());

                let texture_index = registry.add_image(image.clone());

                registry.register_texture(
                    Block::new(
                        block_id,
                        PackedData::builder()
                            .with_bool(false)
                            .with_material(material)
                            .with_bool(false)
                            .build(),
                    ),
                    texture_index,
                );

                for grass_material in Material::iter() {
                    if grass_material.kind() == MaterialKind::Grass {
                        let mut overlay = grass_side_image.clone();
                        grass_material.color_image(overlay.as_mut_rgba8().unwrap());

                        let mut image = image.clone();

                        for x in 0..overlay.width() {
                            for y in 0..overlay.height() {
                                let pixel = overlay.get_pixel(x, y);
                                if pixel.0[3] > 0 {
                                    image.put_pixel(x, y, pixel);
                                }
                            }
                        }

                        let texture_index = registry.add_image(image);

                        registry.register_texture(
                            Block::new(
                                block_id,
                                PackedData::builder()
                                    .with_bool(false)
                                    .with_material(material)
                                    .with_bool(true)
                                    .with_material(grass_material)
                                    .build(),
                            ),
                            texture_index,
                        );
                    }
                }
            }
        }
    }

    fn face_data(&self, face: BlockFace, data: PackedData) -> PackedData {
        let mut data = data.decode();

        let material = data.take_material();
        let is_grass = data.take_bool();

        if !is_grass {
            return PackedData::builder()
                .with_bool(false)
                .with_material(material)
                .with_bool(false)
                .build();
        }

        let grass_material = data.take_material();

        match face {
            BlockFace::Top => PackedData::builder()
                .with_bool(true)
                .with_material(grass_material)
                .build(),
            BlockFace::Bottom => PackedData::builder()
                .with_bool(false)
                .with_material(material)
                .with_bool(false)
                .build(),
            _ => PackedData::builder()
                .with_bool(false)
                .with_material(material)
                .with_bool(true)
                .with_material(grass_material)
                .build(),
        }
    }
}
