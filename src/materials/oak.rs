use image::Rgba;

use crate::{Material, extract_palette};

pub struct Oak;

impl Material for Oak {
    fn unique_name(&self) -> String {
        "oak".to_string()
    }

    fn tags(&self) -> Vec<String> {
        vec!["wood".to_string()]
    }

    fn get_palette(&self) -> [Rgba<u8>; 4] {
        let image =
            image::load_from_memory(include_bytes!("../../textures/materials/oak.png")).unwrap();
        extract_palette(&image)
    }
}
