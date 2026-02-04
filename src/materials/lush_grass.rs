use image::Rgba;

use crate::{Material, extract_palette};

pub struct LushGrass;

impl Material for LushGrass {
    fn unique_name(&self) -> String {
        "lush_grass".to_string()
    }

    fn tags(&self) -> Vec<String> {
        vec!["grass".to_string()]
    }

    fn get_palette(&self) -> [Rgba<u8>; 4] {
        let image =
            image::load_from_memory(include_bytes!("../../textures/materials/lush_grass.png"))
                .unwrap();
        extract_palette(&image)
    }
}
