use image::{ImageBuffer, Rgba};
use num_derive::{FromPrimitive, ToPrimitive};
use strum::{Display, EnumIter};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Display)]
pub enum MaterialKind {
    #[strum(to_string = "Rock")]
    Rock,

    #[strum(to_string = "Soil")]
    Soil,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Display, EnumIter, ToPrimitive, FromPrimitive,
)]
#[repr(u16)]
pub enum Material {
    #[strum(to_string = "Basalt")]
    Shale,

    #[strum(to_string = "Chalk")]
    Chalk,

    #[strum(to_string = "Loam")]
    Loam,

    #[strum(to_string = "Silt")]
    Silt,

    #[strum(to_string = "Clay")]
    Clay,
}

impl Material {
    pub fn kind(&self) -> MaterialKind {
        match self {
            Self::Shale => MaterialKind::Rock,
            Self::Chalk => MaterialKind::Rock,
            Self::Loam => MaterialKind::Soil,
            Self::Silt => MaterialKind::Soil,
            Self::Clay => MaterialKind::Soil,
        }
    }

    pub fn palette(&self) -> [Rgba<u8>; 4] {
        match self {
            Self::Shale => [
                Rgba([0x9D, 0x9D, 0x9D, 0xFF]),
                Rgba([0x83, 0x83, 0x83, 0xFF]),
                Rgba([0x71, 0x71, 0x71, 0xFF]),
                Rgba([0x5B, 0x5B, 0x5B, 0xFF]),
            ],
            Self::Chalk => [
                Rgba([0xE5, 0xE5, 0xD8, 0xFF]),
                Rgba([0xD8, 0xD8, 0xCC, 0xFF]),
                Rgba([0xCC, 0xCC, 0xC0, 0xFF]),
                Rgba([0xC0, 0xC0, 0xB5, 0xFF]),
            ],
            Self::Loam => [
                Rgba([0x82, 0x6C, 0x4A, 0xFF]),
                Rgba([0x76, 0x62, 0x44, 0xFF]),
                Rgba([0x6B, 0x59, 0x3E, 0xFF]),
                Rgba([0x60, 0x50, 0x38, 0xFF]),
            ],
            Self::Silt => [
                Rgba([0xAF, 0x92, 0x6A, 0xFF]),
                Rgba([0xA3, 0x88, 0x62, 0xFF]),
                Rgba([0x98, 0x7E, 0x5A, 0xFF]),
                Rgba([0x8D, 0x74, 0x53, 0xFF]),
            ],
            Self::Clay => [
                Rgba([0xA0, 0x5E, 0x4C, 0xFF]),
                Rgba([0x94, 0x56, 0x45, 0xFF]),
                Rgba([0x88, 0x4E, 0x3E, 0xFF]),
                Rgba([0x7D, 0x47, 0x38, 0xFF]),
            ],
        }
    }

    pub fn color_image(&self, image: &mut ImageBuffer<Rgba<u8>, Vec<u8>>) {
        let palette = self.palette();
        let base_palette = base_palette_colors();

        for pixel in image.pixels_mut() {
            let Some(index) = base_palette.iter().position(|p| p == pixel) else {
                continue;
            };

            *pixel = palette[index];
        }
    }
}

fn base_palette_colors() -> [Rgba<u8>; 4] {
    [
        Rgba([0x9D, 0x9D, 0x9D, 0xFF]),
        Rgba([0x83, 0x83, 0x83, 0xFF]),
        Rgba([0x71, 0x71, 0x71, 0xFF]),
        Rgba([0x5B, 0x5B, 0x5B, 0xFF]),
    ]
}
