use image::Rgba;

use crate::{Material, extract_palette};

pub struct Loam;

impl Material for Loam {
    fn unique_name(&self) -> String {
        "loam".to_string()
    }

    fn tags(&self) -> Vec<String> {
        vec!["soil".to_string()]
    }

    fn get_palette(&self) -> [Rgba<u8>; 4] {
        let image =
            image::load_from_memory(include_bytes!("../../textures/materials/loam.png")).unwrap();
        extract_palette(&image)
    }
}
