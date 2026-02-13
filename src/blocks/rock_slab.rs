use bevy::prelude::*;

use crate::{Aabb, Block, BlockType, PackedData, Registry, color_image};

pub struct RockSlab;

impl BlockType for RockSlab {
    fn unique_name(&self) -> String {
        "rock_slab".to_string()
    }

    fn get_aabb(&self, _data: PackedData) -> Option<Aabb> {
        Some(Aabb::new(Vec3::ZERO, Vec3::new(1.0, 0.5, 1.0)))
    }

    fn model_name(&self) -> &str {
        "slab"
    }

    fn register(&self, registry: &mut Registry) {
        let block_id = registry.block_id(&self.unique_name());
        // Use the same rock texture as the regular Rock block
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
