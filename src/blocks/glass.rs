use crate::{Block, BlockType, PackedData, Registry};

pub struct Glass;

impl BlockType for Glass {
    fn unique_name(&self) -> String {
        "glass".to_string()
    }

    fn register(&self, registry: &mut Registry) {
        let block_id = registry.block_id(&self.unique_name());
        let image =
            image::load_from_memory(include_bytes!("../../textures/blocks/glass.png")).unwrap();

        let texture_index = registry.add_image(image);

        registry.register_texture(
            Block::new(block_id, PackedData::builder().build()),
            texture_index,
        );
    }
}
