use crate::{
    Block, BlockFace, BlockType, PackedData, Registry, RenderContext, color_image, overlay_image,
};

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

    fn render(&self, ctx: &mut RenderContext) {
        let model_id = self.model_id(ctx.registry, ctx.block.data);

        let (top_texture, bottom_texture, side_texture) = self.texture_set(ctx, ctx.block.data);

        for face in BlockFace::ALL {
            let texture = match face {
                BlockFace::Top => top_texture,
                BlockFace::Bottom => bottom_texture,
                BlockFace::Front | BlockFace::Back | BlockFace::Left | BlockFace::Right => {
                    side_texture
                }
            };

            ctx.add_model_face(model_id, face, texture, false);
        }
    }
}

impl Soil {
    fn texture_set(&self, ctx: &RenderContext, data: PackedData) -> (u32, u32, u32) {
        let mut data = data.decode();

        let material = data.take_material();
        let is_grass = data.take_bool();

        if !is_grass {
            let texture_data = PackedData::builder()
                .with_bool(false)
                .with_material(material)
                .with_bool(false)
                .build();

            let texture = ctx.texture_index_for_data(texture_data);

            return (texture, texture, texture);
        }

        let grass_material = data.take_material();
        let top = PackedData::builder()
            .with_bool(true)
            .with_material(grass_material)
            .build();
        let bottom = PackedData::builder()
            .with_bool(false)
            .with_material(material)
            .with_bool(false)
            .build();
        let side = PackedData::builder()
            .with_bool(false)
            .with_material(material)
            .with_bool(true)
            .with_material(grass_material)
            .build();

        (
            ctx.texture_index_for_data(top),
            ctx.texture_index_for_data(bottom),
            ctx.texture_index_for_data(side),
        )
    }
}
