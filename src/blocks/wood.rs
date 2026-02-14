use crate::{Block, BlockFace, BlockType, PackedData, Registry, RenderContext, color_image};

pub struct Wood;

impl BlockType for Wood {
    fn unique_name(&self) -> String {
        "wood".to_string()
    }

    fn register(&self, registry: &mut Registry) {
        let block_id = registry.block_id(&self.unique_name());
        let wood_top_image =
            image::load_from_memory(include_bytes!("../../textures/blocks/wood_top.png")).unwrap();
        let wood_side_image =
            image::load_from_memory(include_bytes!("../../textures/blocks/wood_side.png")).unwrap();

        for id in registry.materials() {
            let material = registry.material(id);

            if !material.tags().contains(&"wood".to_string()) {
                continue;
            }

            let top_image = color_image(&wood_top_image, material.get_palette());
            let side_image = color_image(&wood_side_image, material.get_palette());

            let top_texture_index = registry.add_image(top_image);
            let side_texture_index = registry.add_image(side_image);

            registry.register_texture(
                Block::new(
                    block_id,
                    PackedData::builder()
                        .with_bool(true)
                        .with_material(id)
                        .build(),
                ),
                top_texture_index,
            );

            registry.register_texture(
                Block::new(
                    block_id,
                    PackedData::builder()
                        .with_bool(false)
                        .with_material(id)
                        .build(),
                ),
                side_texture_index,
            );
        }
    }

    fn render(&self, ctx: &mut RenderContext) {
        let mut data = ctx.block.data.decode();
        let material = data.take_material();

        let top_data = PackedData::builder()
            .with_bool(true)
            .with_material(material)
            .build();
        let side_data = PackedData::builder()
            .with_bool(false)
            .with_material(material)
            .build();

        let top_texture = ctx.texture_index_for_data(top_data);
        let side_texture = ctx.texture_index_for_data(side_data);
        let model_id = self.model_id(ctx.registry, ctx.block.data);

        for face in BlockFace::ALL {
            let texture = if matches!(face, BlockFace::Top | BlockFace::Bottom) {
                top_texture
            } else {
                side_texture
            };

            ctx.add_model_face(model_id, face, texture, false);
        }
    }
}
