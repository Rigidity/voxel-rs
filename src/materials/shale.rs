use image::Rgba;

use crate::{Material, extract_palette};

pub struct Shale;

impl Material for Shale {
    fn unique_name(&self) -> String {
        "shale".to_string()
    }

    fn tags(&self) -> Vec<String> {
        vec!["rock".to_string()]
    }

    fn get_palette(&self) -> [Rgba<u8>; 4] {
        let image =
            image::load_from_memory(include_bytes!("../../textures/materials/shale.png")).unwrap();
        extract_palette(&image)
    }
}
