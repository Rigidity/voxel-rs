use std::num::NonZero;

use crate::Texture;

pub struct TextureArray {
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
}

impl TextureArray {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
        let dirt_bytes = include_bytes!("../textures/Dirt.png");
        let dirt_texture = Texture::from_bytes(device, queue, dirt_bytes, "Dirt.png").unwrap();

        let rock_bytes = include_bytes!("../textures/Rock.png");
        let rock_texture = Texture::from_bytes(device, queue, rock_bytes, "Rock.png").unwrap();

        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: NonZero::new(2),
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
            label: Some("texture_array_bind_group_layout"),
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureViewArray(&[
                        &dirt_texture.view,
                        &rock_texture.view,
                    ]),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&dirt_texture.sampler),
                },
            ],
            layout: &layout,
            label: Some("texture_array_bind_group"),
        });

        Self {
            bind_group_layout: layout,
            bind_group,
        }
    }
}
