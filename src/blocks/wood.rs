use crate::{Block, BlockFace, BlockType, PackedData, Registry, color_image};

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

    fn face_data(&self, face: BlockFace, data: PackedData) -> PackedData {
        let mut data = data.decode();

        let material = data.take_material();

        if matches!(face, BlockFace::Top | BlockFace::Bottom) {
            return PackedData::builder()
                .with_bool(true)
                .with_material(material)
                .build();
        }

        PackedData::builder()
            .with_bool(false)
            .with_material(material)
            .build()
    }
}
