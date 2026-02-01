use std::{collections::HashMap, sync::Arc};

use bevy::{
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};
use strum::IntoEnumIterator;

use crate::{BlockKind, ChunkMaterial, TextureArrayBuilder};

pub struct RegistryPlugin;

impl Plugin for RegistryPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<ChunkMaterial>::default())
            .add_systems(Startup, setup_registry);
    }
}

#[derive(Resource)]
pub struct BlockTextureArray {
    pub handle: Handle<Image>,
    pub material: Handle<ChunkMaterial>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockTextureKey {
    pub block_kind: BlockKind,
    pub texture_data: u64,
}

impl BlockTextureKey {
    pub fn new(block_kind: BlockKind, texture_data: u64) -> Self {
        Self {
            block_kind,
            texture_data,
        }
    }
}

#[derive(Resource)]
pub struct SharedRegistry(pub Arc<Registry>);

#[derive(Default)]
pub struct Registry {
    block_texture_indices: HashMap<BlockTextureKey, u32>,
}

impl Registry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register_texture(
        &mut self,
        block_kind: BlockKind,
        texture_data: u64,
        texture_index: u32,
    ) {
        self.block_texture_indices.insert(
            BlockTextureKey::new(block_kind, texture_data),
            texture_index,
        );
    }

    pub fn texture_index(&self, block_kind: BlockKind, texture_data: u64) -> u32 {
        self.block_texture_indices[&BlockTextureKey::new(block_kind, texture_data)]
    }
}

fn setup_registry(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<ChunkMaterial>>,
) {
    let mut registry = Registry::new();
    let mut builder = TextureArrayBuilder::new(16, 16);

    let atlas = image::load_from_memory(include_bytes!("../../textures/textures.png")).unwrap();

    for block_kind in BlockKind::iter() {
        block_kind.register_textures(&mut builder, &mut registry, &atlas);
    }

    let textures = builder.into_textures();

    log::info!("Generating an array of {} textures", textures.len());

    // Convert Vec<DynamicImage> into a texture array
    let texture_size = 16u32;
    let array_layers = textures.len() as u32;

    let mut texture_array_data = Vec::new();

    for texture in textures {
        let rgba = texture.to_rgba8();
        texture_array_data.extend_from_slice(&rgba);
    }

    let texture_array = Image::new(
        Extent3d {
            width: texture_size,
            height: texture_size,
            depth_or_array_layers: array_layers,
        },
        TextureDimension::D2,
        texture_array_data,
        TextureFormat::Rgba8UnormSrgb,
        Default::default(),
    );

    let handle = images.add(texture_array);

    let material = materials.add(ChunkMaterial {
        array_texture: handle.clone(),
        ao_factor: 0.3,
    });

    commands.insert_resource(SharedRegistry(Arc::new(registry)));
    commands.insert_resource(BlockTextureArray { handle, material });
}
