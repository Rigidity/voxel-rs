use image::DynamicImage;

#[derive(Debug, Clone)]
pub struct TextureArrayBuilder {
    textures: Vec<DynamicImage>,
    width: u32,
    height: u32,
}

impl TextureArrayBuilder {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            textures: Vec::new(),
            width,
            height,
        }
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
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
