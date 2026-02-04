use crate::{Block, BlockFace, BlockType, PackedData, Registry, color_image, overlay_image};

pub struct Soil;

impl BlockType for Soil {
    fn unique_name(&self) -> String {
        "soil".to_string()
    }

    fn register(&self, registry: &mut Registry) {
        let block_id = registry.block_id(&self.unique_name());
        let soil_image =
            image::load_from_memory(include_bytes!("../../textures/blocks/soil.png")).unwrap();
        let grass_side_image =
            image::load_from_memory(include_bytes!("../../textures/blocks/grass.png")).unwrap();

        for id in registry.materials() {
            let material = registry.material(id);

            if !material.tags().contains(&"grass".to_string()) {
                continue;
            }

            let image = color_image(&soil_image, material.get_palette());
            let texture_index = registry.add_image(image);

            registry.register_texture(
                Block::new(
                    block_id,
                    PackedData::builder()
                        .with_bool(true)
                        .with_material(id)
                        .build(),
                ),
                texture_index,
            );
        }

        for soil_id in registry.materials() {
            let soil_material = registry.material(soil_id);

            if !soil_material.tags().contains(&"soil".to_string()) {
                continue;
            }

            let image = color_image(&soil_image, soil_material.get_palette());
            let texture_index = registry.add_image(image.clone());

            registry.register_texture(
                Block::new(
                    block_id,
                    PackedData::builder()
                        .with_bool(false)
                        .with_material(soil_id)
                        .with_bool(false)
                        .build(),
                ),
                texture_index,
            );

            for grass_id in registry.materials() {
                let grass_material = registry.material(grass_id);

                if !grass_material.tags().contains(&"grass".to_string()) {
                    continue;
                }

                let overlay = color_image(&grass_side_image, grass_material.get_palette());
                let image = overlay_image(&image, &overlay);
                let texture_index = registry.add_image(image);

                registry.register_texture(
                    Block::new(
                        block_id,
                        PackedData::builder()
                            .with_bool(false)
                            .with_material(soil_id)
                            .with_bool(true)
                            .with_material(grass_id)
                            .build(),
                    ),
                    texture_index,
                );
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
