use glam::IVec3;
use indexmap::IndexMap;
use oneshot::TryRecvError;
use wgpu::util::DeviceExt;

use crate::{CHUNK_SIZE, ChunkMesh, ChunkVertex, RelevantChunks, Texture, World};

pub struct VoxelPipeline {
    pipeline: wgpu::RenderPipeline,
    texture_bind_group: wgpu::BindGroup,
    chunk_position_bind_group_layout: wgpu::BindGroupLayout,
    chunk_meshes: IndexMap<IVec3, ChunkMesh>,
    mesh_tasks: IndexMap<IVec3, oneshot::Receiver<Option<ChunkMesh>>>,
}

impl VoxelPipeline {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        texture_format: wgpu::TextureFormat,
        camera_bind_group_layout: &wgpu::BindGroupLayout,
    ) -> Self {
        let diffuse_bytes = include_bytes!("../textures/Dirt.png");
        let diffuse_texture =
            Texture::from_bytes(device, queue, diffuse_bytes, "Dirt.png").unwrap();

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: Some("texture_bind_group_layout"),
            });

        let diffuse_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&diffuse_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&diffuse_texture.sampler),
                },
            ],
            label: Some("diffuse_bind_group"),
        });

        let chunk_position_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("chunk_position_bind_group_layout"),
            });

        let shader = device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[
                &texture_bind_group_layout,
                camera_bind_group_layout,
                &chunk_position_bind_group_layout,
            ],
            immediate_size: 0,
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[ChunkVertex::descriptor()],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: texture_format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: Texture::DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview_mask: None,
            cache: None,
        });

        Self {
            pipeline,
            texture_bind_group: diffuse_bind_group,
            chunk_position_bind_group_layout,
            chunk_meshes: IndexMap::new(),
            mesh_tasks: IndexMap::new(),
        }
    }

    pub fn tick(&mut self, device: &wgpu::Device, world: &mut World) {
        self.chunk_meshes
            .retain(|chunk_pos, _| world.chunks.contains_key(chunk_pos));

        self.mesh_tasks
            .retain(|chunk_pos, _| world.chunks.contains_key(chunk_pos));

        let mut chunks_to_mesh = world
            .chunks
            .keys()
            .copied()
            .filter(|chunk_pos| {
                world.chunks[chunk_pos].is_dirty() && !self.mesh_tasks.contains_key(chunk_pos)
            })
            .collect::<Vec<_>>();

        chunks_to_mesh.sort_by_key(|chunk_pos| world.chunk_sort_key(*chunk_pos));

        for chunk_pos in chunks_to_mesh {
            if self.mesh_tasks.len() >= rayon::current_num_threads() {
                break;
            }

            let neighbors = [
                chunk_pos - IVec3::X,
                chunk_pos + IVec3::X,
                chunk_pos - IVec3::Y,
                chunk_pos + IVec3::Y,
                chunk_pos - IVec3::Z,
                chunk_pos + IVec3::Z,
            ];

            let mut should_generate = true;

            for neighbor in neighbors {
                should_generate &=
                    world.chunks.contains_key(&neighbor) || !world.is_visible_chunk(neighbor);
            }

            if !should_generate {
                continue;
            }

            let (sender, receiver) = oneshot::channel();

            let relevant_chunks = RelevantChunks::from_world(world, chunk_pos);

            let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Chunk Position Buffer"),
                contents: bytemuck::cast_slice(&[ChunkUniform::new(chunk_pos)]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

            let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &self.chunk_position_bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buffer.as_entire_binding(),
                }],
                label: Some("chunk_position_bind_group"),
            });

            let device = device.clone();

            rayon::spawn(move || {
                let mesh =
                    ChunkMesh::from_chunk_data(&device, chunk_pos, &relevant_chunks, bind_group);
                sender.send(mesh).ok();
            });

            self.mesh_tasks.insert(chunk_pos, receiver);
        }

        self.mesh_tasks
            .retain(|chunk_pos, receiver| match receiver.try_recv() {
                Ok(mesh) => {
                    if let Some(chunk) = world.chunks.get_mut(chunk_pos) {
                        if let Some(mesh) = mesh {
                            self.chunk_meshes.insert(*chunk_pos, mesh);
                        }

                        chunk.clear_dirty();
                    }

                    false
                }
                Err(TryRecvError::Disconnected) => false,
                Err(TryRecvError::Empty) => true,
            });
    }

    pub fn render(&self, render_pass: &mut wgpu::RenderPass, camera_bind_group: &wgpu::BindGroup) {
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.texture_bind_group, &[]);
        render_pass.set_bind_group(1, camera_bind_group, &[]);

        for mesh in self.chunk_meshes.values() {
            mesh.draw(render_pass);
        }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct ChunkUniform {
    chunk_position: [f32; 3],
}

impl ChunkUniform {
    fn new(chunk_position: IVec3) -> Self {
        Self {
            chunk_position: [
                chunk_position.x as f32 * CHUNK_SIZE as f32,
                chunk_position.y as f32 * CHUNK_SIZE as f32,
                chunk_position.z as f32 * CHUNK_SIZE as f32,
            ],
        }
    }
}
