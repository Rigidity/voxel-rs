use crate::{Block, BlockType, PackedData, Registry, color_image};

pub struct Rock;

impl BlockType for Rock {
    fn unique_name(&self) -> String {
        "rock".to_string()
    }

    fn register(&self, registry: &mut Registry) {
        let block_id = registry.block_id(&self.unique_name());
        let image =
            image::load_from_memory(include_bytes!("../../textures/blocks/rock.png")).unwrap();

        for id in registry.materials() {
            let material = registry.material(id);

            if !material.tags().contains(&"rock".to_string()) {
                continue;
            }

            let image = color_image(&image, material.get_palette());
            let texture_index = registry.add_image(image);

            registry.register_texture(
                Block::new(block_id, PackedData::builder().with_material(id).build()),
                texture_index,
            );
        }
    }
}
