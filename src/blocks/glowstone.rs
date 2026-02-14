use crate::{BlockType, PackedData, Registry};

pub struct Glowstone;

impl BlockType for Glowstone {
    fn unique_name(&self) -> String {
        "glowstone".to_string()
    }

    fn register(&self, registry: &mut Registry) {
        let block_id = registry.block_id(&self.unique_name());
        let image =
            image::load_from_memory(include_bytes!("../../textures/blocks/glowstone.png"))
                .unwrap();
        let texture_index = registry.add_image(image);

        registry.register_texture(
            crate::Block::new(block_id, PackedData::builder().build()),
            texture_index,
        );
    }

    fn light_emission(&self, _data: PackedData) -> u8 {
        15
    }
}
