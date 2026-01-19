use anyhow::Result;
use winit::event_loop::{ControlFlow, EventLoop};

mod app;
mod app_state;
mod block;
mod camera;
mod chunk;
mod level;
mod texture;
mod voxel_mesh;
mod voxel_renderer;

pub use app::*;
pub use app_state::*;
pub use block::*;
pub use camera::*;
pub use chunk::*;
pub use level::*;
pub use texture::*;
pub use voxel_mesh::*;
pub use voxel_renderer::*;

fn main() -> Result<()> {
    env_logger::init();

    let event_loop = EventLoop::new()?;

    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::new();

    event_loop.run_app(&mut app).unwrap();

    Ok(())
}
