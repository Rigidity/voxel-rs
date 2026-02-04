use image::{DynamicImage, GenericImageView, Rgba};

pub trait Material: 'static + Send + Sync {
    fn unique_name(&self) -> String;

    fn tags(&self) -> Vec<String>;

    fn get_palette(&self) -> [Rgba<u8>; 4];
}

pub fn color_image(image: &DynamicImage, palette: [Rgba<u8>; 4]) -> DynamicImage {
    let mut image = image.to_rgba8();

    let base_palette = base_palette_colors();

    for pixel in image.pixels_mut() {
        let Some(index) = base_palette.iter().position(|p| p == pixel) else {
            continue;
        };

        *pixel = palette[index];
    }

    image.into()
}

pub fn overlay_image(image: &DynamicImage, overlay: &DynamicImage) -> DynamicImage {
    let mut image = image.to_rgba8();

    for x in 0..overlay.width() {
        for y in 0..overlay.height() {
            let pixel = overlay.get_pixel(x, y);

            if pixel.0[3] > 0 {
                image.put_pixel(x, y, pixel);
            }
        }
    }

    image.into()
}

pub fn extract_palette(image: &DynamicImage) -> [Rgba<u8>; 4] {
    image
        .as_rgba8()
        .unwrap()
        .pixels()
        .copied()
        .collect::<Vec<_>>()
        .try_into()
        .unwrap()
}

fn base_palette_colors() -> [Rgba<u8>; 4] {
    [
        Rgba([0x9D, 0x9D, 0x9D, 0xFF]),
        Rgba([0x83, 0x83, 0x83, 0xFF]),
        Rgba([0x71, 0x71, 0x71, 0xFF]),
        Rgba([0x5B, 0x5B, 0x5B, 0xFF]),
    ]
}
