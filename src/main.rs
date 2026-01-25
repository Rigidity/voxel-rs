use anyhow::Result;
use winit::event_loop::{ControlFlow, EventLoop};

mod aabb;
mod app;
mod app_state;
mod block;
mod camera;
mod chunk;
mod chunk_data;
mod level;
mod player;
mod projection;
mod texture;
mod voxel_mesh;
mod voxel_renderer;
mod window;
mod world_generator;

pub use aabb::*;
pub use app::*;
pub use app_state::*;
pub use block::*;
pub use camera::*;
pub use chunk::*;
pub use chunk_data::*;
pub use level::*;
pub use player::*;
pub use projection::*;
pub use texture::*;
pub use voxel_mesh::*;
pub use voxel_renderer::*;
pub use window::*;
pub use world_generator::*;

fn main() -> Result<()> {
    env_logger::init();

    let event_loop = EventLoop::new()?;

    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::new();

    event_loop.run_app(&mut app).unwrap();

    Ok(())
}
