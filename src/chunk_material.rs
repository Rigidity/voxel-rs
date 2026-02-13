use bevy::{
    mesh::{MeshVertexAttribute, MeshVertexBufferLayoutRef, VertexFormat},
    pbr::{MaterialPipeline, MaterialPipelineKey},
    prelude::*,
    render::{
        render_resource::{AsBindGroup, RenderPipelineDescriptor, SpecializedMeshPipelineError},
        storage::ShaderStorageBuffer,
    },
    shader::ShaderRef,
};

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct ChunkMaterial {
    #[texture(0, dimension = "2d_array")]
    #[sampler(1)]
    pub array_texture: Handle<Image>,
    #[uniform(2)]
    pub ao_factor: f32,
    #[storage(3, read_only)]
    pub model_buffer: Handle<ShaderStorageBuffer>,
}

pub const ATTRIBUTE_PACKED_DATA: MeshVertexAttribute =
    MeshVertexAttribute::new("PackedData", 47198478, VertexFormat::Uint32);

pub const ATTRIBUTE_TEXTURE_INDEX: MeshVertexAttribute =
    MeshVertexAttribute::new("TextureIndex", 47198479, VertexFormat::Uint32);

impl Material for ChunkMaterial {
    fn vertex_shader() -> ShaderRef {
        "shader.wgsl".into()
    }

    fn fragment_shader() -> ShaderRef {
        "shader.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::AlphaToCoverage
    }

    fn specialize(
        _pipeline: &MaterialPipeline,
        descriptor: &mut RenderPipelineDescriptor,
        layout: &MeshVertexBufferLayoutRef,
        _key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        let vertex_layout = layout.0.get_layout(&[
            ATTRIBUTE_PACKED_DATA.at_shader_location(0),
            ATTRIBUTE_TEXTURE_INDEX.at_shader_location(1),
        ])?;
        descriptor.vertex.buffers = vec![vertex_layout];
        Ok(())
    }
}
