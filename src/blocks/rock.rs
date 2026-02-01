use strum::IntoEnumIterator;

use crate::{Block, BlockType, Material, MaterialKind, PackedData, Registry};

pub struct Rock;

impl BlockType for Rock {
    fn unique_name(&self) -> String {
        "rock".to_string()
    }

    fn register(&self, registry: &mut Registry) {
        let block_id = registry.block_id(&self.unique_name());
        let image = image::load_from_memory(include_bytes!("../../textures/rock.png")).unwrap();

        for material in Material::iter() {
            if material.kind() == MaterialKind::Rock {
                let mut image = image.clone().into_rgba8();

                material.color_image(&mut image);

                let texture_index = registry.add_image(image.into());

                registry.register_texture(
                    Block::new(
                        block_id,
                        PackedData::builder().with_material(material).build(),
                    ),
                    texture_index,
                );
            }
        }
    }
}
