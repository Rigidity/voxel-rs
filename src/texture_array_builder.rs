use image::DynamicImage;

#[derive(Debug, Default, Clone)]
pub struct TextureArrayBuilder {
    textures: Vec<DynamicImage>,
}

impl TextureArrayBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_bytes(&mut self, bytes: &[u8]) -> u32 {
        let image = image::load_from_memory(bytes).unwrap();
        self.add_image(image)
    }

    pub fn add_image(&mut self, img: DynamicImage) -> u32 {
        let texture_index = self.textures.len() as u32;
        self.textures.push(img);
        texture_index
    }

    pub fn into_textures(self) -> Vec<DynamicImage> {
        self.textures
    }
}
