use image::{ImageBuffer, Rgba};
use num_derive::{FromPrimitive, ToPrimitive};
use strum::{Display, EnumIter};

macro_rules! color {
    ( $( $color:tt )* ) => {
        csscolorparser::parse(stringify!($( $color )*)).unwrap().to_rgba8()
    };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Display)]
pub enum MaterialKind {
    #[strum(to_string = "Rock")]
    Rock,

    #[strum(to_string = "Soil")]
    Soil,

    #[strum(to_string = "Grass")]
    Grass,
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

    #[strum(to_string = "Clay")]
    Clay,

    #[strum(to_string = "Lush Grass")]
    LushGrass,
}

impl Material {
    pub fn kind(&self) -> MaterialKind {
        match self {
            Self::Shale => MaterialKind::Rock,
            Self::Chalk => MaterialKind::Rock,
            Self::Loam => MaterialKind::Soil,
            Self::Clay => MaterialKind::Soil,
            Self::LushGrass => MaterialKind::Grass,
        }
    }

    pub fn palette(&self) -> [Rgba<u8>; 4] {
        match self {
            Self::Shale => [
                Rgba(color!( #949494 )),
                Rgba(color!( #838383 )),
                Rgba(color!( #737373 )),
                Rgba(color!( #656565 )),
            ],
            Self::Chalk => [
                Rgba(color!( #E5E5D8 )),
                Rgba(color!( #D8D8CC )),
                Rgba(color!( #CCC0C0 )),
                Rgba(color!( #C0C0B5 )),
            ],
            Self::Loam => [
                Rgba(color!(rgb(114, 89, 55))),
                Rgba(color!(rgb(110, 84, 49))),
                Rgba(color!(rgb(103, 79, 45))),
                Rgba(color!(rgb(97, 72, 42))),
            ],
            Self::Clay => [
                Rgba(color!( #A05E4C )),
                Rgba(color!( #945645 )),
                Rgba(color!( #884E3E )),
                Rgba(color!( #7D4738 )),
            ],
            Self::LushGrass => [
                Rgba(color!(rgb(48, 162, 54))),
                Rgba(color!(rgb(45, 155, 51))),
                Rgba(color!(rgb(43, 148, 48))),
                Rgba(color!(rgb(40, 139, 44))),
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
