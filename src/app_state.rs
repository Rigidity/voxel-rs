use std::{sync::Arc, time::Instant};

use glam::Vec3;
use wgpu::util::DeviceExt;
use winit::{keyboard::KeyCode, window::Window};

use crate::{Camera, Input, Level, Projection, Texture, VoxelRenderer};

pub struct AppState {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    is_surface_configured: bool,
    voxel_renderer: VoxelRenderer,
    depth_texture: Texture,
    camera: Camera,
    projection: Projection,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    level: Level,
    window: Arc<Window>,
    input: Input,
    // FPS tracking
    frame_count: u32,
    last_fps_print_time: Instant,
}

impl AppState {
    pub async fn new(window: Arc<Window>) -> anyhow::Result<Self> {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });

        let surface = instance.create_surface(window.clone())?;

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await?;

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                experimental_features: wgpu::ExperimentalFeatures::disabled(),
                required_limits: wgpu::Limits::default(),
                memory_hints: Default::default(),
                trace: wgpu::Trace::Off,
            })
            .await?;

        let surface_caps = surface.get_capabilities(&adapter);

        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        let camera = Camera::new(Vec3::new(0.0, 0.0, 640.0), -90.0, 0.0);
        let projection = Projection::new(config.width, config.height, 60.0, 0.1, 1000.0);
        let camera_uniform = CameraUniform::new(&camera, &projection);

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let camera_bind_group_layout =
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
                label: Some("camera_bind_group_layout"),
            });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
            label: Some("camera_bind_group"),
        });

        let depth_texture = Texture::create_depth_texture(&device, &config, "depth_texture");

        let voxel_renderer =
            VoxelRenderer::new(&device, &queue, config.format, &camera_bind_group_layout);
        let level = Level::new(&device, &voxel_renderer);

        Ok(Self {
            surface,
            device,
            queue,
            config,
            is_surface_configured: false,
            voxel_renderer,
            depth_texture,
            camera,
            projection,
            camera_buffer,
            camera_bind_group,
            level,
            window,
            input: Input::default(),
            frame_count: 0,
            last_fps_print_time: Instant::now(),
        })
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    pub fn set_key_pressed(&mut self, key: KeyCode, pressed: bool) {
        self.input.set_key_pressed(key, pressed);
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.config.width = width;
            self.config.height = height;
            self.surface.configure(&self.device, &self.config);
            self.is_surface_configured = true;
        }

        self.depth_texture =
            Texture::create_depth_texture(&self.device, &self.config, "depth_texture");
    }

    pub fn update(&mut self) {
        self.level.update(&self.device);
        self.camera.update(&self.input);

        let camera_uniform = CameraUniform::new(&self.camera, &self.projection);

        self.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[camera_uniform]),
        );
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        self.window.request_redraw();

        if !self.is_surface_configured {
            return Ok(());
        }

        let output = self.surface.get_current_texture()?;

        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.1,
                        g: 0.2,
                        b: 0.3,
                        a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
                depth_slice: None,
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &self.depth_texture.view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            occlusion_query_set: None,
            timestamp_writes: None,
            multiview_mask: None,
        });

        self.voxel_renderer
            .render(&mut render_pass, &self.camera_bind_group, &self.level);

        drop(render_pass);

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        // FPS calculation
        self.frame_count += 1;
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_fps_print_time);

        if elapsed.as_secs() >= 1 {
            let fps = self.frame_count as f64 / elapsed.as_secs_f64();
            println!("FPS: {:.2}", fps);
            self.frame_count = 0;
            self.last_fps_print_time = now;
        }

        Ok(())
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct CameraUniform {
    view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
    fn new(camera: &Camera, projection: &Projection) -> Self {
        let view = camera.view_matrix();
        let projection = projection.projection_matrix();
        Self {
            view_proj: (projection * view).to_cols_array_2d(),
        }
    }
}
