use std::{collections::HashMap, sync::Arc};

use bevy::{
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};
use image::DynamicImage;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use crate::{
    Block, BlockFace, BlockType, ChunkMaterial, Loam, LushGrass, Material, Oak, Rock, Shale, Soil,
    Wood,
};

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct BlockId(pub u16);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct MaterialId(pub u16);

#[derive(Resource)]
pub struct SharedRegistry(pub Arc<Registry>);

#[derive(Default)]
pub struct Registry {
    material_ids: IndexMap<String, MaterialId>,
    materials: HashMap<MaterialId, Box<dyn Material>>,
    block_ids: HashMap<String, BlockId>,
    block_types: HashMap<BlockId, Box<dyn BlockType>>,
    block_texture_indices: HashMap<Block, u32>,
    texture_array: Vec<DynamicImage>,
}

impl Registry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn material_id(&self, name: &str) -> MaterialId {
        self.material_ids[name]
    }

    pub fn material(&self, id: MaterialId) -> &dyn Material {
        &*self.materials[&id]
    }

    pub fn materials(&self) -> Vec<MaterialId> {
        self.material_ids.values().copied().collect()
    }

    pub fn register_material(&mut self, material: impl Material) {
        let material_id = MaterialId(self.materials.len() as u16);
        self.material_ids
            .insert(material.unique_name(), material_id);
        self.materials.insert(material_id, Box::new(material));
    }

    pub fn register_block(&mut self, block: impl BlockType) {
        let block_id = BlockId(self.block_ids.len() as u16);
        self.block_ids.insert(block.unique_name(), block_id);
        block.register(self);
        self.block_types.insert(block_id, Box::new(block));
    }

    pub fn block_id(&self, name: &str) -> BlockId {
        self.block_ids[name]
    }

    pub fn block_type(&self, id: BlockId) -> &dyn BlockType {
        &*self.block_types[&id]
    }

    pub fn add_image(&mut self, image: DynamicImage) -> u32 {
        let texture_index = self.texture_array.len() as u32;
        self.texture_array.push(image);
        texture_index
    }

    pub fn register_texture(&mut self, block: Block, texture_index: u32) {
        self.block_texture_indices.insert(block, texture_index);
    }

    pub fn texture_index(&self, block: Block, face: BlockFace) -> u32 {
        let data = self.block_type(block.id).face_data(face, block.data);
        self.block_texture_indices[&Block::new(block.id, data)]
    }
}

fn setup_registry(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<ChunkMaterial>>,
) {
    let mut registry = Registry::new();

    registry.register_material(Loam);
    registry.register_material(LushGrass);
    registry.register_material(Oak);
    registry.register_material(Shale);

    registry.register_block(Rock);
    registry.register_block(Soil);
    registry.register_block(Wood);

    log::info!(
        "Generating an array of {} textures",
        registry.texture_array.len()
    );

    // Convert Vec<DynamicImage> into a texture array
    let texture_size = 16u32;
    let array_layers = registry.texture_array.len() as u32;

    let mut texture_array_data = Vec::new();

    for texture in &registry.texture_array {
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
